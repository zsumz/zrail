//! Field access facts share exact receiver identity without counting place writes as reads.

use syn::{Expr, ExprField, Member};

use super::{
    SourceOperationKind,
    fact::source_span,
    operation_model::{TypeIdentity, unresolved, unwrapped},
    place_expression::PlaceExpression,
    visitor::FactVisitor,
};

impl FactVisitor<'_> {
    pub(super) fn record_field_read(&mut self, field: &ExprField) {
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

    pub(super) fn with_place_operation(
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

    fn record_field(&mut self, kind: SourceOperationKind, field: &ExprField) {
        let Member::Named(member) = &field.member else {
            return;
        };
        let receiver = self.field_receiver(&field.base);
        let identity = TypeIdentity {
            name: format!("{}::{member}", receiver.name),
            quality: receiver.quality,
            file_local: receiver.file_local,
        };
        self.push_operation(kind, &identity, member.to_string(), Some(member.span()));
    }

    fn field_receiver(&self, expression: &Expr) -> TypeIdentity {
        match unwrapped(expression) {
            Expr::Path(path) if path.path.is_ident("self") => self
                .self_types
                .last()
                .cloned()
                .unwrap_or_else(|| unresolved("self")),
            _ => unresolved("<unresolved>"),
        }
    }
}
