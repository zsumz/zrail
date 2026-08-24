//! Written and physically resolved path identity are retained together.

use syn::{
    ExprPath, Path,
    spanned::Spanned,
    visit::{self, Visit as _},
};

use super::{fact::written_fact, visitor::FactVisitor};

impl FactVisitor<'_> {
    pub(super) fn record_expression_path(&mut self, expression: &ExprPath) {
        let previous =
            std::mem::replace(&mut self.next_path_namespace, super::FactNamespace::Value);
        if let Some(boundary) =
            super::calls::unresolved_path_projection(expression, self.syntax_guard())
        {
            self.call_resolutions.push(boundary);
            for attribute in &expression.attrs {
                self.visit_attribute(attribute);
            }
            if let Some(qself) = &expression.qself {
                self.visit_qself(qself);
            }
            for segment in &expression.path.segments {
                self.visit_path_arguments(&segment.arguments);
            }
        } else {
            visit::visit_expr_path(self, expression);
        }
        self.next_path_namespace = previous;
    }

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

#[cfg(test)]
#[path = "visitor_paths_test.rs"]
mod visitor_paths_test;
