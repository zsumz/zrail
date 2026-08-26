//! Shared syntax operations retain only identities the source can prove.

#[path = "visitor_operations/identity.rs"]
mod identity;

use std::collections::BTreeMap;

use syn::{Expr, ExprCall, ExprPath, ExprStruct, Item, Type};
use zrail_core::AnalysisQuality;

use super::{
    FactVisitor, SourceOperationFact, SourceOperationKind,
    fact::{fact, written_fact},
    operation_model::{
        ConstructorForm, TypeIdentity, append, last_segment_looks_constructor, local_type,
        path_text, unresolved,
    },
};

impl FactVisitor<'_> {
    pub(in crate::source) fn with_local_type_scope<'a>(
        &mut self,
        items: impl Iterator<Item = &'a Item>,
        visit: impl FnOnce(&mut Self),
    ) {
        let prefix = self.inline_modules.join("::");
        let types = items
            .filter_map(|item| local_type(item, &prefix))
            .collect::<BTreeMap<_, _>>();
        self.local_types.push(types);
        visit(self);
        self.local_types.pop();
    }

    pub(in crate::source) fn with_inline_module(
        &mut self,
        name: String,
        visit: impl FnOnce(&mut Self),
    ) {
        self.inline_modules.push(name);
        visit(self);
        self.inline_modules.pop();
    }

    pub(in crate::source) fn with_self_type(&mut self, ty: &Type, visit: impl FnOnce(&mut Self)) {
        let identity = self.resolve_type(ty);
        self.self_types.push(identity);
        visit(self);
        self.self_types.pop();
    }

    pub(in crate::source) fn record_struct_construction(&mut self, expression: &ExprStruct) {
        let identity = self.resolve_identity(&expression.path);
        self.push_operation(
            SourceOperationKind::TypeConstruction,
            &identity,
            path_text(&expression.path),
            expression
                .path
                .segments
                .last()
                .map(|segment| segment.ident.span()),
        );
    }

    pub(in crate::source) fn record_call_construction(&mut self, call: &ExprCall) {
        let Expr::Path(callee) = call.func.as_ref() else {
            return;
        };
        let Some((form, proven)) = self.constructor_form(&callee.path) else {
            return;
        };
        if form != ConstructorForm::Tuple {
            return;
        }
        let mut identity = self.resolve_identity(&callee.path);
        if !proven {
            identity.quality = AnalysisQuality::Unresolved;
        }
        self.push_operation(
            SourceOperationKind::TypeConstruction,
            &identity,
            path_text(&callee.path),
            callee
                .path
                .segments
                .last()
                .map(|segment| segment.ident.span()),
        );
    }

    pub(in crate::source) fn record_path_construction(&mut self, expression: &ExprPath) {
        let exact = self.constructor_form(&expression.path) == Some((ConstructorForm::Unit, true));
        if !exact && !last_segment_looks_constructor(&expression.path) {
            return;
        }
        let mut identity = self.resolve_identity(&expression.path);
        if !exact {
            identity.quality = AnalysisQuality::Unresolved;
        }
        self.push_operation(
            SourceOperationKind::TypeConstruction,
            &identity,
            path_text(&expression.path),
            expression
                .path
                .segments
                .last()
                .map(|segment| segment.ident.span()),
        );
    }

    pub(in crate::source) fn record_method_operation(&mut self, call: &syn::ExprMethodCall) {
        let identity = TypeIdentity {
            name: call.method.to_string(),
            quality: AnalysisQuality::Exact,
            file_local: false,
            span: Some(super::fact::source_span(call.method.span())),
        };
        self.push_operation(
            SourceOperationKind::MethodCall,
            &identity,
            call.method.to_string(),
            Some(call.method.span()),
        );
    }

    pub(in crate::source) fn push_operation(
        &mut self,
        kind: SourceOperationKind,
        identity: &TypeIdentity,
        written: String,
        span: Option<proc_macro2::Span>,
    ) {
        self.push_operation_with_method(kind, identity, written, span, None, None, None);
    }

    pub(in crate::source) fn push_field_receiver_operation(
        &mut self,
        identity: &TypeIdentity,
        written: String,
        span: Option<proc_macro2::Span>,
        method: String,
        place: Option<super::operation_model::FieldPlaceFact>,
        guard: &super::SyntaxGuard,
    ) {
        self.push_operation_with_method(
            SourceOperationKind::FieldReceiverCall,
            identity,
            written,
            span,
            Some(method),
            place,
            Some(guard),
        );
    }

    pub(in crate::source) fn push_field_operation(
        &mut self,
        kind: SourceOperationKind,
        identity: &TypeIdentity,
        written: String,
        span: Option<proc_macro2::Span>,
        place: Option<super::operation_model::FieldPlaceFact>,
        guard: &super::SyntaxGuard,
    ) {
        self.push_operation_with_method(kind, identity, written, span, None, place, Some(guard));
    }

    fn push_operation_with_method(
        &mut self,
        kind: SourceOperationKind,
        identity: &TypeIdentity,
        written: String,
        span: Option<proc_macro2::Span>,
        method: Option<String>,
        place: Option<super::operation_model::FieldPlaceFact>,
        guard: Option<&super::SyntaxGuard>,
    ) {
        let mut observed = span.map_or_else(
            || {
                fact(
                    &identity.name,
                    proc_macro2::Span::call_site(),
                    identity.quality,
                )
            },
            |span| {
                written_fact(
                    &identity.name,
                    written,
                    span,
                    identity.quality,
                    &self.lexical_scope,
                )
            },
        );
        if let Some(guard) = guard {
            observed.apply_guard(guard);
        }
        observed.namespace = super::FactNamespace::Type;
        if self.operations.iter().any(|operation| {
            operation.kind == kind
                && operation.identity.name == observed.name
                && operation.identity.span == observed.span
                && operation.method == method
                && operation.identity.guard == observed.guard
        }) {
            return;
        }
        self.operations.push(SourceOperationFact {
            kind,
            identity: observed,
            file_local: identity.file_local,
            method,
            place,
        });
    }
}

#[cfg(test)]
#[path = "visitor_operations_test.rs"]
mod visitor_operations_test;
