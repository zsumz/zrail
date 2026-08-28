//! Operation facts share one guarded, duplicate-aware emission path.

use super::super::{
    FactVisitor, SourceOperationFact, SourceOperationKind,
    fact::{fact, written_fact},
    operation_model::{FieldPlaceFact, QualifiedOperationSubject, StructUpdateFact, TypeIdentity},
};
use super::ConstructorCandidate;
use super::ConstructorForm;

#[derive(Default)]
struct OperationDetails<'a> {
    construction: Option<ConstructorForm>,
    construction_proven: bool,
    method: Option<String>,
    place: Option<FieldPlaceFact>,
    struct_update: Option<StructUpdateFact>,
    qualified_subject: Option<QualifiedOperationSubject>,
    root_lookup: Option<super::super::RootLookupNamespace>,
    guard: Option<&'a super::super::SyntaxGuard>,
}

impl FactVisitor<'_> {
    pub(in crate::source) fn push_operation(
        &mut self,
        kind: SourceOperationKind,
        identity: &TypeIdentity,
        written: String,
        span: Option<proc_macro2::Span>,
        construction: Option<ConstructorForm>,
    ) {
        self.push_operation_with_method(
            kind,
            identity,
            written,
            span,
            OperationDetails {
                construction,
                construction_proven: construction == Some(ConstructorForm::Named),
                root_lookup: construction.map(|_| super::super::RootLookupNamespace::Type),
                ..OperationDetails::default()
            },
        );
    }

    pub(in crate::source) fn push_guarded_constructor_candidate(
        &mut self,
        identity: &TypeIdentity,
        written: String,
        span: Option<proc_macro2::Span>,
        guard: &super::super::SyntaxGuard,
        candidate: ConstructorCandidate,
    ) {
        self.push_operation_with_method(
            candidate.kind,
            identity,
            written,
            span,
            OperationDetails {
                construction: Some(candidate.form),
                construction_proven: candidate.proven,
                root_lookup: Some(candidate.root_lookup),
                guard: Some(guard),
                qualified_subject: candidate.qualified_subject,
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
        place: Option<FieldPlaceFact>,
        guard: &super::super::SyntaxGuard,
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
        place: Option<FieldPlaceFact>,
        guard: &super::super::SyntaxGuard,
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
        place: FieldPlaceFact,
        update: StructUpdateFact,
        rest_span: proc_macro2::Span,
        guard: &super::super::SyntaxGuard,
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
            construction,
            construction_proven,
            method,
            place,
            struct_update,
            qualified_subject,
            root_lookup,
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
        observed.namespace = match root_lookup {
            Some(super::super::RootLookupNamespace::Value) => super::super::FactNamespace::Value,
            Some(super::super::RootLookupNamespace::Type) | None => {
                super::super::FactNamespace::Type
            }
        };
        let generic_shadow = root_lookup.and_then(|lookup| {
            let written = observed.written.as_deref().unwrap_or(&observed.name);
            super::super::generic_root_shadow(
                written,
                lookup,
                &self.generic_types,
                &self.generic_values,
            )
        });
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
            root_lookup,
            generic_shadow,
            file_local: identity.file_local,
            subject_origin: identity.origin,
            construction,
            construction_proven,
            method,
            place,
            struct_update,
            qualified_subject,
        });
    }
}
