//! Macro identity uses exact lexical imports before conservative file-wide candidates.

use std::collections::BTreeMap;

use syn::{Item, Path};
use zrail_core::AnalysisQuality;

use super::{scoped_imports, visitor::FactVisitor};

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

pub(super) type LocalImportScopes = Vec<BTreeMap<String, scoped_imports::ScopedAlias>>;

fn split_root(path: &str) -> (&str, &str) {
    path.find("::").map_or((path, ""), |separator| {
        (&path[..separator], &path[separator..])
    })
}
