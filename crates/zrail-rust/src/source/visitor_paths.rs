//! Written and physically resolved path identity are retained together.

use syn::{Path, spanned::Spanned};

use super::{fact::written_fact, visitor::FactVisitor};

impl FactVisitor<'_> {
    pub(super) fn record_path(&mut self, path: &Path) {
        let guard = self.syntax_guard();
        let (name, quality) = self.imports.resolve(path, guard);
        if name.is_empty() {
            return;
        }
        let written = path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>()
            .join("::");
        self.paths.push(written_fact(
            name.as_str(),
            written,
            path.span(),
            quality,
            &self.lexical_scope,
        ));
        self.paths
            .extend(super::calls::candidates(path, self.imports, &name, guard));
    }
}
