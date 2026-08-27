//! Shared syntax operations retain only identities the source can prove.

#[path = "visitor_operations/construction.rs"]
mod construction;
#[path = "visitor_operations/identity.rs"]
mod identity;

use std::collections::BTreeMap;

use syn::{Item, Type};
use zrail_core::AnalysisQuality;

use super::{
    FactVisitor, SourceOperationFact, SourceOperationKind,
    fact::{fact, written_fact},
    operation_model::{
        ConstructorForm, TypeIdentity, append, last_segment_looks_constructor, local_type,
        path_text, unresolved,
    },
};

#[derive(Default)]
struct OperationDetails<'a> {
    exact_construction_syntax: bool,
    method: Option<String>,
    place: Option<super::operation_model::FieldPlaceFact>,
    struct_update: Option<super::operation_model::StructUpdateFact>,
    guard: Option<&'a super::SyntaxGuard>,
}

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
            false,
        );
    }

    pub(in crate::source) fn push_operation(
        &mut self,
        kind: SourceOperationKind,
        identity: &TypeIdentity,
        written: String,
        span: Option<proc_macro2::Span>,
        exact_construction_syntax: bool,
    ) {
        self.push_operation_with_method(
            kind,
            identity,
            written,
            span,
            OperationDetails {
                exact_construction_syntax,
                ..OperationDetails::default()
            },
        );
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
            OperationDetails {
                method: Some(method),
                place,
                guard: Some(guard),
                ..OperationDetails::default()
            },
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
        self.push_operation_with_method(
            kind,
            identity,
            written,
            span,
            OperationDetails {
                place,
                guard: Some(guard),
                ..OperationDetails::default()
            },
        );
    }

    pub(in crate::source) fn push_deferred_struct_update(
        &mut self,
        identity: &TypeIdentity,
        place: super::operation_model::FieldPlaceFact,
        update: super::operation_model::StructUpdateFact,
        rest_span: proc_macro2::Span,
        guard: &super::SyntaxGuard,
    ) {
        self.push_operation_with_method(
            SourceOperationKind::FieldRead,
            identity,
            "*".into(),
            Some(rest_span),
            OperationDetails {
                place: Some(place),
                struct_update: Some(update),
                guard: Some(guard),
                ..OperationDetails::default()
            },
        );
    }

    fn push_operation_with_method(
        &mut self,
        kind: SourceOperationKind,
        identity: &TypeIdentity,
        written: String,
        span: Option<proc_macro2::Span>,
        details: OperationDetails<'_>,
    ) {
        let OperationDetails {
            exact_construction_syntax,
            method,
            place,
            struct_update,
            guard,
        } = details;
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
            exact_construction_syntax,
            method,
            place,
            struct_update,
        });
    }
}

#[cfg(test)]
#[path = "visitor_operations_test.rs"]
mod visitor_operations_test;
