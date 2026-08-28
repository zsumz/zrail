//! Written and physically resolved path identity are retained together.

use syn::{
    ExprPath, Path,
    spanned::Spanned,
    visit::{self, Visit as _},
};

use super::{
    FactVisitor,
    fact::{written_fact, written_path},
};

impl FactVisitor<'_> {
    pub(in crate::source) fn record_expression_path(&mut self, expression: &ExprPath) {
        let previous =
            std::mem::replace(&mut self.next_path_namespace, super::FactNamespace::Value);
        if let Some(boundary) = super::calls::unresolved_path_projection(
            expression,
            self.syntax_guard(),
            &self.generic_types,
        ) {
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

    pub(in crate::source) fn record_path(&mut self, path: &Path) {
        let guard = self.syntax_guard();
        let (name, quality) = self.imports.resolve(path, &guard);
        if name.is_empty() {
            return;
        }
        let written = written_path(path);
        let mut fact = written_fact(
            name.as_str(),
            written,
            path.span(),
            quality,
            &self.lexical_scope,
        );
        fact.namespace = std::mem::take(&mut self.next_path_namespace);
        let value_position = fact.namespace == super::FactNamespace::Value;
        let scoped = self.with_implicit_prelude_scope(std::iter::once(fact), value_position);
        let generic_root = scoped.iter().any(|fact| fact.generic_shadow.is_some());
        self.paths.extend(scoped);
        if !generic_root {
            self.paths
                .extend(super::calls::candidates(path, self.imports, &name, &guard));
        }
    }
}

#[cfg(test)]
#[path = "visitor_paths_test.rs"]
mod visitor_paths_test;
