//! Macro identity uses exact lexical imports before conservative file-wide candidates.

#[path = "visitor_import_scope.rs"]
mod scope;

use std::collections::BTreeMap;

use syn::{Path, spanned::Spanned};
use zrail_core::AnalysisQuality;

use super::{
    FactVisitor, MacroCandidate, MacroDerivation, MacroExpansionFact, SyntaxGuard, fact::fact,
    scoped_imports,
};

const MAX_MACRO_CANDIDATES: usize = 64;

impl FactVisitor<'_> {
    pub(in crate::source) fn resolve_macro_path(
        &self,
        path: &Path,
    ) -> (String, AnalysisQuality, bool, bool) {
        let text = path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>()
            .join("::");
        let (target, quality, scoped, local_module, _) = self.resolve_text_scoped(&text);
        (target, quality, scoped, local_module)
    }

    pub(in crate::source) fn macro_invocation(&self, path: &Path) -> MacroExpansionFact {
        let written_name = path_text(path);
        let (resolved, quality, scoped, local_module) = self.resolve_macro_path(path);
        let mut observed = fact(resolved.clone(), path.span(), quality);
        if local_module {
            observed.canonical.push(resolved.clone());
        }
        let derivation = if self.imports.re_exports(path, &self.syntax_guard()) {
            MacroDerivation::ReExport
        } else if resolved != written_name {
            MacroDerivation::ExactImport
        } else if local_module {
            MacroDerivation::LocalDefinition
        } else {
            MacroDerivation::Written
        };
        let mut candidates = vec![MacroCandidate::pending(observed, local_module, derivation)];
        if !scoped {
            let (imported, overflowed) =
                super::calls::macro_candidates(path, self.imports, &resolved, &self.syntax_guard());
            if overflowed {
                let unresolved = fact(&written_name, path.span(), AnalysisQuality::Unresolved);
                return MacroExpansionFact::with_candidates(
                    fact(written_name, path.span(), AnalysisQuality::Unresolved),
                    vec![MacroCandidate::unresolved(
                        unresolved,
                        MacroDerivation::GlobImport,
                    )],
                )
                .with_lexical_scope(&self.lexical_scope);
            }
            candidates.extend(imported.into_iter().map(|(observed, derivation)| {
                MacroCandidate::pending(observed, false, derivation)
            }));
            candidates.extend(self.local_macro_candidates(path, &resolved));
        }
        candidates.sort_by(|left, right| left.observation.name.cmp(&right.observation.name));
        candidates.dedup_by(|left, right| left.observation.name == right.observation.name);
        if candidates.len() > MAX_MACRO_CANDIDATES {
            let unresolved = fact(&written_name, path.span(), AnalysisQuality::Unresolved);
            return MacroExpansionFact::with_candidates(
                unresolved.clone(),
                vec![MacroCandidate::unresolved(
                    unresolved,
                    MacroDerivation::GlobImport,
                )],
            )
            .with_lexical_scope(&self.lexical_scope);
        }
        MacroExpansionFact::with_candidates(fact(written_name, path.span(), quality), candidates)
            .with_lexical_scope(&self.lexical_scope)
    }

    fn local_macro_candidates(&self, path: &Path, resolved: &str) -> Vec<MacroCandidate> {
        let written = path_text(path);
        let mut candidates = BTreeMap::new();
        for scope in &self.local_imports {
            for (glob, guard) in &scope.globs {
                if !guard.overlaps(self.syntax_guard()) {
                    continue;
                }
                let (resolved_glob, _, _, _, _) = self.resolve_text_scoped(glob);
                let name = format!("{resolved_glob}::{written}");
                if name != resolved {
                    super::import_helpers::insert_guard(&mut candidates, name, guard);
                }
            }
        }
        candidates
            .into_iter()
            .map(|(name, guard)| {
                let mut observed = fact(name, path.span(), AnalysisQuality::Conservative);
                observed.guard = guard;
                MacroCandidate::pending(observed, false, MacroDerivation::GlobImport)
            })
            .collect()
    }

    pub(in crate::source) fn resolve_text(&self, path: &str) -> scoped_imports::ScopedAlias {
        let (target, quality, _, local_module, guard) = self.resolve_text_scoped(path);
        scoped_imports::ScopedAlias {
            target,
            quality,
            local_module,
            guard,
        }
    }

    pub(in crate::source) fn resolve_text_scoped(
        &self,
        path: &str,
    ) -> (String, AnalysisQuality, bool, bool, super::SyntaxGuard) {
        let (root, suffix) = split_root(path);
        for scope in self.local_imports.iter().rev() {
            if let Some(alias) = scope.aliases.get(root) {
                let availability = alias.guard.availability_in(self.syntax_guard());
                if !availability.is_available() {
                    continue;
                }
                let quality =
                    alias
                        .quality
                        .max(if availability == super::GuardAvailability::Possible {
                            AnalysisQuality::Unresolved
                        } else {
                            AnalysisQuality::Exact
                        });
                if !suffix.is_empty() && visible_root(&alias.target) == visible_root(root) {
                    if alias.local_module {
                        return (path.into(), quality, true, true, alias.guard.clone());
                    }
                    continue;
                }
                return (
                    format!("{}{suffix}", alias.target),
                    quality,
                    true,
                    alias.local_module,
                    alias.guard.clone(),
                );
            }
        }
        let Ok(parsed) = syn::parse_str::<Path>(path) else {
            return (
                path.into(),
                AnalysisQuality::Unresolved,
                false,
                false,
                super::SyntaxGuard::Ordinary,
            );
        };
        let (resolved, quality, guard) = self
            .imports
            .resolve_with_guard(&parsed, &self.syntax_guard());
        (resolved, quality, false, false, guard)
    }
}

fn visible_root(path: &str) -> &str {
    let root = path.split("::").next().unwrap_or(path);
    root.strip_prefix("r#").unwrap_or(root)
}

#[derive(Debug)]
pub(in crate::source) struct LocalImportScope {
    aliases: BTreeMap<String, scoped_imports::ScopedAlias>,
    globs: BTreeMap<String, SyntaxGuard>,
}

pub(in crate::source) type LocalImportScopes = Vec<LocalImportScope>;

fn split_root(path: &str) -> (&str, &str) {
    path.find("::").map_or((path, ""), |separator| {
        (&path[..separator], &path[separator..])
    })
}

fn path_text(path: &Path) -> String {
    path.segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

#[cfg(test)]
#[path = "visitor_imports_test.rs"]
mod visitor_imports_test;
