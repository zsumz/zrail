//! Named field syntax always emits an operation; exact identity requires a proven declaration.

use syn::{Expr, ExprField, ExprMethodCall, Member, Path};

use super::{
    FactVisitor, SourceOperationKind, SyntaxGuard,
    fact::source_span,
    operation_model::{FieldPlaceFact, unwrapped},
    place_expression::PlaceExpression,
};

#[path = "visitor_field_candidates.rs"]
mod candidates;

impl FactVisitor<'_> {
    pub(in crate::source) fn record_field_read(&mut self, field: &ExprField) {
        let Member::Named(member) = &field.member else {
            return;
        };
        if self
            .field_read_exclusions
            .contains(&source_span(member.span()))
        {
            return;
        }
        self.record_field(SourceOperationKind::FieldRead, field);
    }

    pub(in crate::source) fn with_place_operation(
        &mut self,
        kind: SourceOperationKind,
        expression: &Expr,
        visit: impl FnOnce(&mut Self),
    ) {
        let place = PlaceExpression::analyze(expression);
        for field in place.authority_fields() {
            self.record_field(kind, field);
        }
        let checkpoint = self.field_read_exclusions.len();
        self.field_read_exclusions.extend(place.excluded_reads());
        visit(self);
        self.field_read_exclusions.truncate(checkpoint);
    }

    pub(in crate::source) fn record_field_receiver_call(&mut self, call: &ExprMethodCall) {
        let Expr::Field(field) = unwrapped(&call.receiver) else {
            return;
        };
        let Member::Named(member) = &field.member else {
            return;
        };
        for context in candidates::field_contexts(self, field) {
            self.push_field_receiver_operation(
                &context.identity,
                member.to_string(),
                Some(member.span()),
                call.method.to_string(),
                context.place,
                &context.guard,
            );
        }
    }

    pub(in crate::source) fn record_pattern_field(
        &mut self,
        path: &Path,
        member: &Member,
        guard: &SyntaxGuard,
    ) {
        let Member::Named(member) = member else {
            return;
        };
        let base = self.resolve_identity(path);
        let identity = candidates::declared_field_identity(self, &base, &member.to_string());
        let place = FieldPlaceFact {
            base_name: base.name,
            base_quality: base.quality,
            base_file_local: base.file_local,
            base_span: base.span,
            fields: vec![member.to_string()],
        };
        self.push_field_operation(
            SourceOperationKind::FieldRead,
            &identity,
            member.to_string(),
            Some(member.span()),
            Some(place),
            guard,
        );
    }

    fn record_field(&mut self, kind: SourceOperationKind, field: &ExprField) {
        let Member::Named(member) = &field.member else {
            return;
        };
        for context in candidates::field_contexts(self, field) {
            self.push_field_operation(
                kind,
                &context.identity,
                member.to_string(),
                Some(member.span()),
                context.place,
                &context.guard,
            );
        }
    }
}
