//! Import aliases and glob roots for syntax-derived path resolution.

use std::collections::{BTreeMap, BTreeSet};

use syn::Item;
use zrail_core::AnalysisQuality;

use super::MacroImportFact;

#[derive(Clone, Debug, Default)]
pub(super) struct ImportMap {
    pub(super) aliases: BTreeMap<String, String>,
    call_aliases: BTreeMap<String, BTreeSet<String>>,
    pub(super) unresolved: BTreeSet<String>,
    pub(super) globs: Vec<String>,
    pub(super) re_exports: BTreeSet<String>,
    pub(super) re_export_globs: BTreeSet<String>,
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
        for item in &file.items {
            match item {
                Item::Use(item) => super::imports_collect::collect_use(
                    &mut imports,
                    Vec::new(),
                    &item.tree,
                    super::scoped_imports::conditional(&item.attrs),
                    !matches!(item.vis, syn::Visibility::Inherited),
                ),
                Item::ExternCrate(item) => {
                    let alias = item
                        .rename
                        .as_ref()
                        .map_or_else(|| item.ident.to_string(), |(_, name)| name.to_string());
                    imports
                        .aliases
                        .insert(alias.clone(), item.ident.to_string());
                    if super::scoped_imports::conditional(&item.attrs) {
                        imports.unresolved.insert(alias);
                    }
                }
                _ => {}
            }
        }
        let candidates = super::import_candidates::collect(file);
        imports.call_aliases = candidates.aliases;
        imports.globs.extend(candidates.globs);
        imports.normalize_aliases();
        super::import_candidates::normalize(&mut imports.call_aliases, &imports.aliases);
        imports.globs.sort();
        imports.globs.dedup();
        imports
    }

    pub(super) fn resolve(&self, path: &syn::Path) -> (String, AnalysisQuality) {
        let segments = path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>();
        let Some(first) = segments.first() else {
            return (String::new(), AnalysisQuality::Unresolved);
        };
        if let Some(prefix) = self.aliases.get(first) {
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
            return (resolved, quality);
        }
        (segments.join("::"), AnalysisQuality::Exact)
    }

    pub(super) fn declared_paths(&self) -> Vec<(&str, AnalysisQuality)> {
        let mut paths = self
            .aliases
            .iter()
            .map(|(alias, path)| {
                let quality = if self.unresolved.contains(alias) {
                    AnalysisQuality::Unresolved
                } else {
                    AnalysisQuality::Exact
                };
                (path.as_str(), quality)
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
                re_export: self.re_exports.contains(name),
            })
            .collect()
    }

    pub(super) fn globs(&self) -> &[String] {
        &self.globs
    }

    pub(super) fn call_candidates(&self, path: &syn::Path) -> Vec<ImportCandidate> {
        self.collect_candidates(path, usize::MAX).0
    }

    pub(super) fn bounded_call_candidates(
        &self,
        path: &syn::Path,
        limit: usize,
    ) -> (Vec<ImportCandidate>, bool) {
        self.collect_candidates(path, limit)
    }

    fn collect_candidates(&self, path: &syn::Path, limit: usize) -> (Vec<ImportCandidate>, bool) {
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
        for prefix in self.call_aliases.get(first).into_iter().flatten() {
            let kind = if self.re_exports.contains(first) {
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
        for glob in &self.globs {
            let kind = if self.re_export_globs.contains(glob) {
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

    pub(super) fn re_exports(&self, path: &syn::Path) -> bool {
        path.segments
            .first()
            .is_some_and(|segment| self.re_exports.contains(&segment.ident.to_string()))
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

fn join_path(mut prefix: String, remainder: &[String]) -> String {
    if !remainder.is_empty() {
        prefix.push_str("::");
        prefix.push_str(&remainder.join("::"));
    }
    prefix
}

#[cfg(test)]
#[path = "imports_test.rs"]
mod imports_test;
