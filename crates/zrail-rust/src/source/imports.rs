//! Import aliases and glob roots for syntax-derived path resolution.

use std::collections::{BTreeMap, BTreeSet};

use syn::Item;
use zrail_core::AnalysisQuality;

use super::import_helpers::{insert_guard, insert_primary_alias, visible_root};
use super::{SyntaxGuard, attributes::is_cfg_test};

#[derive(Clone, Debug, Default)]
pub(super) struct ImportMap {
    pub(super) aliases: BTreeMap<String, String>,
    pub(super) alias_guards: BTreeMap<String, SyntaxGuard>,
    pub(super) call_aliases: BTreeMap<String, BTreeMap<String, SyntaxGuard>>,
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
