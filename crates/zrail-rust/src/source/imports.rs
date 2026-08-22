//! Import aliases and glob roots for syntax-derived path resolution.

use std::collections::{BTreeMap, BTreeSet};

use syn::Item;
use zrail_core::AnalysisQuality;

use super::import_helpers::{insert_guard, insert_primary_alias, join_path, visible_root};
use super::{MacroImportFact, SyntaxGuard, attributes::is_cfg_test};

#[derive(Clone, Debug, Default)]
pub(super) struct ImportMap {
    pub(super) aliases: BTreeMap<String, String>,
    pub(super) alias_guards: BTreeMap<String, SyntaxGuard>,
    call_aliases: BTreeMap<String, BTreeMap<String, SyntaxGuard>>,
    pub(super) unresolved: BTreeSet<String>,
    pub(super) globs: BTreeMap<String, SyntaxGuard>,
    pub(super) re_exports: BTreeMap<String, SyntaxGuard>,
    pub(super) re_export_globs: BTreeMap<String, SyntaxGuard>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ImportCandidateKind {
    Exact,
    Glob,
    ReExport,
}

pub(super) struct ImportCandidate {
    pub(super) path: String,
    pub(super) kind: ImportCandidateKind,
}

impl ImportMap {
    pub(super) fn from_file(file: &syn::File) -> Self {
        let mut imports = Self::default();
        let file_guard = SyntaxGuard::for_test_only(file.attrs.iter().any(is_cfg_test));
        for item in &file.items {
            match item {
                Item::Use(item) => {
                    let guard = file_guard.combine(SyntaxGuard::for_test_only(
                        item.attrs.iter().any(is_cfg_test),
                    ));
                    super::imports_collect::collect_use(
                        &mut imports,
                        Vec::new(),
                        &item.tree,
                        super::scoped_imports::conditional(&item.attrs)
                            && guard == SyntaxGuard::Ordinary,
                        guard,
                        !matches!(item.vis, syn::Visibility::Inherited),
                    );
                }
                Item::ExternCrate(item) => {
                    let alias = item
                        .rename
                        .as_ref()
                        .map_or_else(|| item.ident.to_string(), |(_, name)| name.to_string());
                    let guard = file_guard.combine(SyntaxGuard::for_test_only(
                        item.attrs.iter().any(is_cfg_test),
                    ));
                    insert_primary_alias(
                        &mut imports.aliases,
                        &mut imports.alias_guards,
                        &mut imports.unresolved,
                        alias.clone(),
                        item.ident.to_string(),
                        super::scoped_imports::conditional(&item.attrs),
                        guard,
                    );
                    if !matches!(item.vis, syn::Visibility::Inherited) {
                        insert_guard(&mut imports.re_exports, alias, guard);
                    }
                }
                _ => {}
            }
        }
        let candidates = super::import_candidates::collect(file);
        imports.call_aliases = candidates.aliases;
        for (path, guard) in candidates.globs {
            insert_guard(&mut imports.globs, path, guard);
        }
        imports.normalize_aliases();
        super::import_candidates::normalize(
            &mut imports.call_aliases,
            &imports.aliases,
            &imports.alias_guards,
        );
        imports
    }

    pub(super) fn resolve(
        &self,
        path: &syn::Path,
        context: SyntaxGuard,
    ) -> (String, AnalysisQuality) {
        let (path, quality, _) = self.resolve_with_guard(path, context);
        (path, quality)
    }

    pub(super) fn resolve_with_guard(
        &self,
        path: &syn::Path,
        context: SyntaxGuard,
    ) -> (String, AnalysisQuality, SyntaxGuard) {
        let segments = path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>();
        let Some(first) = segments.first() else {
            return (
                String::new(),
                AnalysisQuality::Unresolved,
                SyntaxGuard::Ordinary,
            );
        };
        if let Some(prefix) = self.aliases.get(first) {
            let guard = self.alias_guards.get(first).copied().unwrap_or_default();
            if !guard.available_in(context) {
                return (segments.join("::"), AnalysisQuality::Unresolved, guard);
            }
            if segments.len() > 1 && visible_root(prefix) == visible_root(first) {
                return (segments.join("::"), AnalysisQuality::Exact, guard);
            }
            let mut resolved = prefix.clone();
            if segments.len() > 1 {
                resolved.push_str("::");
                resolved.push_str(&segments[1..].join("::"));
            }
            let quality = if self.unresolved.contains(first) {
                AnalysisQuality::Unresolved
            } else {
                AnalysisQuality::Exact
            };
            return (resolved, quality, guard);
        }
        (
            segments.join("::"),
            AnalysisQuality::Exact,
            SyntaxGuard::Ordinary,
        )
    }

    pub(super) fn declared_paths(&self) -> Vec<(&str, AnalysisQuality, SyntaxGuard)> {
        let mut paths = self
            .aliases
            .iter()
            .map(|(alias, path)| {
                let quality = if self.unresolved.contains(alias) {
                    AnalysisQuality::Unresolved
                } else {
                    AnalysisQuality::Exact
                };
                (
                    path.as_str(),
                    quality,
                    self.alias_guards.get(alias).copied().unwrap_or_default(),
                )
            })
            .collect::<Vec<_>>();
        paths.sort_unstable();
        paths.dedup();
        paths
    }

    pub(super) fn macro_imports(&self) -> Vec<MacroImportFact> {
        self.aliases
            .iter()
            .map(|(name, target)| MacroImportFact {
                name: name.clone(),
                target: target.clone(),
                quality: if self.unresolved.contains(name) {
                    AnalysisQuality::Unresolved
                } else {
                    AnalysisQuality::Exact
                },
                guard: self.alias_guards.get(name).copied().unwrap_or_default(),
                re_export: self.re_exports.contains_key(name),
            })
            .collect()
    }

    pub(super) fn declared_globs(&self) -> Vec<(&str, SyntaxGuard)> {
        self.globs
            .iter()
            .map(|(path, guard)| (path.as_str(), *guard))
            .collect()
    }

    pub(super) fn call_candidates(
        &self,
        path: &syn::Path,
        context: SyntaxGuard,
    ) -> Vec<ImportCandidate> {
        self.collect_candidates(path, usize::MAX, context).0
    }

    pub(super) fn bounded_call_candidates(
        &self,
        path: &syn::Path,
        limit: usize,
        context: SyntaxGuard,
    ) -> (Vec<ImportCandidate>, bool) {
        self.collect_candidates(path, limit, context)
    }

    fn collect_candidates(
        &self,
        path: &syn::Path,
        limit: usize,
        context: SyntaxGuard,
    ) -> (Vec<ImportCandidate>, bool) {
        let segments = path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>();
        let Some(first) = segments.first() else {
            return (Vec::new(), false);
        };
        let remainder = &segments[1..];
        let mut candidates = BTreeMap::new();
        for (prefix, guard) in self.call_aliases.get(first).into_iter().flatten() {
            if !guard.available_in(context) {
                continue;
            }
            let kind = if self
                .re_exports
                .get(first)
                .is_some_and(|guard| guard.available_in(context))
            {
                ImportCandidateKind::ReExport
            } else {
                ImportCandidateKind::Exact
            };
            candidates.insert(join_path(prefix.clone(), remainder), kind);
            if candidates.len() > limit {
                return (Vec::new(), true);
            }
        }
        let syntactic = segments.join("::");
        for (glob, guard) in &self.globs {
            if !guard.available_in(context) {
                continue;
            }
            let kind = if self
                .re_export_globs
                .get(glob)
                .is_some_and(|guard| guard.available_in(context))
            {
                ImportCandidateKind::ReExport
            } else {
                ImportCandidateKind::Glob
            };
            candidates
                .entry(format!("{glob}::{syntactic}"))
                .or_insert(kind);
            if candidates.len() > limit {
                return (Vec::new(), true);
            }
        }
        (
            candidates
                .into_iter()
                .map(|(path, kind)| ImportCandidate { path, kind })
                .collect(),
            false,
        )
    }

    pub(super) fn re_exports(&self, path: &syn::Path, context: SyntaxGuard) -> bool {
        path.segments
            .first()
            .and_then(|segment| self.re_exports.get(&segment.ident.to_string()))
            .is_some_and(|guard| guard.available_in(context))
    }

    fn normalize_aliases(&mut self) {
        let raw = self.aliases.clone();
        let mut cache = BTreeMap::new();
        for alias in raw.keys() {
            let mut visiting = BTreeSet::new();
            match super::import_aliases::expand_alias(alias, &raw, &mut visiting, &mut cache, 0) {
                Some(target) => {
                    self.aliases.insert(alias.clone(), target);
                }
                None => {
                    self.unresolved.insert(alias.clone());
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "imports_test.rs"]
mod imports_test;
