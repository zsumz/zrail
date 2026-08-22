//! Guard-aware policy and call projections derived from collected imports.

use std::collections::BTreeMap;

use zrail_core::AnalysisQuality;

use super::{
    MacroImportFact, SyntaxGuard,
    import_helpers::join_path,
    imports::{ImportCandidate, ImportCandidateKind, ImportMap},
};

impl ImportMap {
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
}
