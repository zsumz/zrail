//! Macro identity uses exact lexical imports before conservative file-wide candidates.

use std::collections::BTreeMap;

use syn::{Item, Path, spanned::Spanned};
use zrail_core::AnalysisQuality;

use super::{
    MacroCandidate, MacroDerivation, MacroExpansionFact, fact::fact, scoped_imports,
    visitor::FactVisitor,
};

impl FactVisitor<'_> {
    pub(super) fn with_import_scope<'a>(
        &mut self,
        items: impl Iterator<Item = &'a Item>,
        visit: impl FnOnce(&mut Self),
    ) {
        let aliases = scoped_imports::collect(items, |path| self.resolve_text(path));
        self.local_imports.push(aliases);
        visit(self);
        self.local_imports.pop();
    }

    pub(super) fn resolve_macro_path(&self, path: &Path) -> (String, AnalysisQuality, bool, bool) {
        let text = path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>()
            .join("::");
        self.resolve_text_scoped(&text)
    }

    pub(super) fn macro_invocation(&self, path: &Path) -> MacroExpansionFact {
        let written_name = path_text(path);
        let (resolved, quality, scoped, local_module) = self.resolve_macro_path(path);
        let mut observed = fact(resolved.clone(), path.span(), quality);
        if local_module {
            observed.canonical.push(resolved.clone());
        }
        let derivation = if self.imports.re_exports(path) {
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
                super::calls::macro_candidates(path, self.imports, &resolved);
            if overflowed {
                let unresolved = fact(&written_name, path.span(), AnalysisQuality::Unresolved);
                return MacroExpansionFact::with_candidates(
                    fact(written_name, path.span(), AnalysisQuality::Unresolved),
                    vec![MacroCandidate::unresolved(
                        unresolved,
                        MacroDerivation::GlobImport,
                    )],
                );
            }
            candidates.extend(imported.into_iter().map(|(observed, derivation)| {
                MacroCandidate::pending(observed, false, derivation)
            }));
        }
        candidates.dedup_by(|left, right| left.observation.name == right.observation.name);
        MacroExpansionFact::with_candidates(fact(written_name, path.span(), quality), candidates)
    }

    fn resolve_text(&self, path: &str) -> scoped_imports::ScopedAlias {
        let (target, quality, _, local_module) = self.resolve_text_scoped(path);
        scoped_imports::ScopedAlias {
            target,
            quality,
            local_module,
        }
    }

    fn resolve_text_scoped(&self, path: &str) -> (String, AnalysisQuality, bool, bool) {
        let (root, suffix) = split_root(path);
        for scope in self.local_imports.iter().rev() {
            if let Some(alias) = scope.get(root) {
                if !suffix.is_empty() && visible_root(&alias.target) == visible_root(root) {
                    if alias.local_module {
                        return (path.into(), alias.quality, true, true);
                    }
                    continue;
                }
                return (
                    format!("{}{suffix}", alias.target),
                    alias.quality,
                    true,
                    alias.local_module,
                );
            }
        }
        let Ok(parsed) = syn::parse_str::<Path>(path) else {
            return (path.into(), AnalysisQuality::Unresolved, false, false);
        };
        let (resolved, quality) = self.imports.resolve(&parsed);
        (resolved, quality, false, false)
    }
}

fn visible_root(path: &str) -> &str {
    let root = path.split("::").next().unwrap_or(path);
    root.strip_prefix("r#").unwrap_or(root)
}

pub(super) type LocalImportScopes = Vec<BTreeMap<String, scoped_imports::ScopedAlias>>;

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
