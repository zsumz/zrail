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
        let mut written = path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>()
            .join("::");
        if path.leading_colon.is_some() {
            written.insert_str(0, "::");
        }
        let mut fact = written_fact(
            name.as_str(),
            written,
            path.span(),
            quality,
            &self.lexical_scope,
        );
        fact.namespace = std::mem::take(&mut self.next_path_namespace);
        self.paths.push(fact);
        self.paths
            .extend(super::calls::candidates(path, self.imports, &name, guard));
    }
}
