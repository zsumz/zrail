//! Written and physically resolved path identity are retained together.

use crate::source::{
    BoundSubject, CallResolutionKind, GenericRootShadow, identity_for_generic_root,
};
use std::collections::BTreeSet;
use syn::{
    ExprPath, Path,
    spanned::Spanned,
    visit::{self, Visit as _},
};

use super::{
    FactNamespace, FactVisitor,
    fact::{written_fact, written_path},
    operation_model::OperationSubjectOrigin,
};

impl FactVisitor<'_> {
    pub(in crate::source) fn record_expression_path(&mut self, expression: &ExprPath) {
        let previous =
            std::mem::replace(&mut self.next_path_namespace, super::FactNamespace::Value);
        if let Some(mut boundary) = super::calls::unresolved_path_projection(
            expression,
            self.syntax_guard(),
            &self.generic_types,
        ) {
            let mut declared = self.generic_types.iter().cloned().collect::<BTreeSet<_>>();
            if self.inherited_generic_roots
                && let Some(qself) = &expression.qself
                && let syn::Type::Path(path) = qself.ty.as_ref()
                && path.qself.is_none()
                && path.path.segments.len() == 1
            {
                declared.insert(path.path.segments[0].ident.to_string());
            }
            let structured = BoundSubject::from_expression(expression, &declared);
            boundary.associated_candidates = structured.as_ref().map_or_else(
                || self.generic_associated_candidates(&boundary.written),
                |(subject, item)| self.generic_associated_candidates_for(subject, item),
            );
            let identity = super::generic_root_identity(
                &boundary.written,
                super::RootLookupNamespace::Type,
                &self.generic_types,
                &self.generic_values,
            )
            .or_else(|| {
                boundary.written.starts_with("Self::").then(|| {
                    identity_for_generic_root(&boundary.written, GenericRootShadow::TypeParameter)
                })
            })
            .or_else(|| {
                structured.as_ref().map(|_| {
                    identity_for_generic_root(&boundary.written, GenericRootShadow::TypeParameter)
                })
            });
            if let Some(identity) = identity {
                let direct_call = self
                    .constructor_path_exclusions
                    .contains(&super::fact::source_span(expression.path.span()));
                let mut fact = written_fact(
                    identity.name,
                    boundary.written,
                    expression.path.span(),
                    identity.quality,
                    &self.lexical_scope,
                );
                fact.namespace = FactNamespace::Value;
                fact.generic_shadow = Some(identity.shadow);
                fact.associated_candidates = boundary.associated_candidates;
                fact.inherits_parent_context = self.inherits_parent_context;
                if direct_call {
                    self.calls.push(fact);
                } else {
                    self.paths.push(fact);
                }
                for attribute in &expression.attrs {
                    self.visit_attribute(attribute);
                }
                for segment in &expression.path.segments {
                    self.visit_path_arguments(&segment.arguments);
                }
                self.next_path_namespace = previous;
                return;
            }
            let contextual =
                boundary.kind == CallResolutionKind::ContextualAssociatedTypeProjection;
            self.call_resolutions.push(boundary);
            if contextual {
                visit::visit_expr_path(self, expression);
                self.next_path_namespace = previous;
                return;
            }
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
        let namespace = std::mem::take(&mut self.next_path_namespace);
        if let Some(mut fact) = self.current_self_fact(path, namespace) {
            fact.inherits_parent_context = self.inherits_parent_context;
            self.paths.push(fact);
            return;
        }
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
        fact.namespace = namespace;
        let value_position = fact.namespace == super::FactNamespace::Value;
        let scoped = self.with_implicit_prelude_scope(std::iter::once(fact), value_position);
        let generic_root = scoped.iter().any(|fact| fact.generic_shadow.is_some());
        self.paths.extend(scoped);
        if !generic_root {
            self.paths
                .extend(super::calls::candidates(path, self.imports, &name, &guard));
        }
    }

    pub(in crate::source) fn current_self_fact(
        &self,
        path: &Path,
        namespace: FactNamespace,
    ) -> Option<super::ObservedFact> {
        if path
            .segments
            .first()
            .is_none_or(|segment| segment.ident != "Self")
        {
            return None;
        }
        if path.segments.len() > 2 {
            return None;
        }
        let identity = self.resolve_identity(path);
        let written = written_path(path);
        let associated_candidates = self.generic_associated_candidates(&written);
        if identity.origin != OperationSubjectOrigin::CurrentSelf {
            let synthetic = identity_for_generic_root(&written, GenericRootShadow::TypeParameter);
            let mut fact = written_fact(
                synthetic.name,
                written,
                path.span(),
                synthetic.quality,
                &self.lexical_scope,
            );
            fact.namespace = namespace;
            fact.generic_shadow = Some(synthetic.shadow);
            fact.associated_candidates = associated_candidates;
            fact.implicit_prelude = super::ImplicitPreludeEligibility::Disabled;
            return Some(fact);
        }
        let mut fact = written_fact(
            identity.name.clone(),
            identity.name,
            path.span(),
            identity.quality,
            &self.lexical_scope,
        );
        fact.namespace = namespace;
        fact.associated_candidates = associated_candidates;
        fact.implicit_prelude = super::ImplicitPreludeEligibility::Disabled;
        Some(fact)
    }
}

#[cfg(test)]
#[path = "visitor_paths_test.rs"]
mod visitor_paths_test;
