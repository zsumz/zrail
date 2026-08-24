//! Field access facts share exact receiver identity without counting place writes as reads.

use syn::{Expr, ExprField, Member};
use zrail_core::SourceSpan;

use super::{
    SourceOperationKind,
    fact::source_span,
    operation_model::{TypeIdentity, field_expression, unresolved, unwrapped},
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

    pub(super) fn record_field_operation(&mut self, kind: SourceOperationKind, expression: &Expr) {
        if let Some(field) = field_expression(expression) {
            self.record_field(kind, field);
        }
    }

    pub(super) fn without_place_field_reads(
        &mut self,
        expression: &Expr,
        visit: impl FnOnce(&mut Self),
    ) {
        let checkpoint = self.field_read_exclusions.len();
        collect_place_fields(expression, &mut self.field_read_exclusions);
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

fn collect_place_fields(expression: &Expr, fields: &mut Vec<SourceSpan>) {
    match expression {
        Expr::Field(field) => {
            if let Member::Named(member) = &field.member {
                fields.push(source_span(member.span()));
            }
            collect_place_fields(&field.base, fields);
        }
        Expr::Index(index) => collect_place_fields(&index.expr, fields),
        Expr::Group(group) => collect_place_fields(&group.expr, fields),
        Expr::Paren(paren) => collect_place_fields(&paren.expr, fields),
        Expr::Tuple(tuple) => {
            for element in &tuple.elems {
                collect_place_fields(element, fields);
            }
        }
        Expr::Array(array) => {
            for element in &array.elems {
                collect_place_fields(element, fields);
            }
        }
        _ => {}
    }
}
