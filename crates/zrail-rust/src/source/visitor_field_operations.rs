//! Field access facts share exact receiver identity without counting place writes as reads.

use syn::{Expr, ExprField, ExprMethodCall, Member, UnOp};

use super::{
    FactVisitor, SourceOperationKind,
    fact::source_span,
    operation_model::{FieldPlaceFact, TypeIdentity, unresolved, unwrapped},
    place_expression::PlaceExpression,
};

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
        let Some(identity) = self.field_identity(field) else {
            return;
        };
        let Some(place) = self.field_place(field) else {
            return;
        };
        let Member::Named(member) = &field.member else {
            return;
        };
        self.push_field_receiver_operation(
            &identity,
            member.to_string(),
            Some(member.span()),
            call.method.to_string(),
            place,
        );
    }

    fn record_field(&mut self, kind: SourceOperationKind, field: &ExprField) {
        let Some(identity) = self.field_identity(field) else {
            return;
        };
        let Some(place) = self.field_place(field) else {
            return;
        };
        let Member::Named(member) = &field.member else {
            return;
        };
        self.push_field_operation(
            kind,
            &identity,
            member.to_string(),
            Some(member.span()),
            place,
        );
    }

    fn field_identity(&self, field: &ExprField) -> Option<TypeIdentity> {
        let Member::Named(member) = &field.member else {
            return None;
        };
        let receiver = self.field_receiver(&field.base);
        Some(TypeIdentity {
            name: format!("{}::{member}", receiver.name),
            quality: receiver.quality,
            file_local: receiver.file_local,
            span: receiver.span,
        })
    }

    fn field_place(&self, field: &ExprField) -> Option<FieldPlaceFact> {
        let Member::Named(member) = &field.member else {
            return None;
        };
        let (base, mut fields) = self.place_base(&field.base)?;
        fields.push(member.to_string());
        Some(FieldPlaceFact {
            base_name: base.name,
            base_quality: base.quality,
            base_file_local: base.file_local,
            base_span: base.span,
            fields,
        })
    }

    fn place_base(&self, expression: &Expr) -> Option<(TypeIdentity, Vec<String>)> {
        match unwrapped(expression) {
            Expr::Path(path) if path.path.is_ident("self") => self
                .self_types
                .last()
                .cloned()
                .map(|base| (base, Vec::new())),
            Expr::Path(path) if path.qself.is_none() && path.path.segments.len() == 1 => self
                .local_value_identity(&path.path.segments[0].ident.to_string())
                .map(|base| (base, Vec::new())),
            Expr::Field(field) => {
                let Member::Named(member) = &field.member else {
                    return None;
                };
                let (base, mut fields) = self.place_base(&field.base)?;
                fields.push(member.to_string());
                Some((base, fields))
            }
            Expr::Unary(unary) if matches!(unary.op, UnOp::Deref(_)) => {
                self.place_base(&unary.expr)
            }
            Expr::Cast(cast) => Some((self.resolve_type(&cast.ty), Vec::new())),
            _ => None,
        }
    }

    fn field_receiver(&self, expression: &Expr) -> TypeIdentity {
        match unwrapped(expression) {
            Expr::Path(path) if path.path.is_ident("self") => self
                .self_types
                .last()
                .cloned()
                .unwrap_or_else(|| unresolved("self")),
            Expr::Path(path) if path.qself.is_none() && path.path.segments.len() == 1 => self
                .local_value_identity(&path.path.segments[0].ident.to_string())
                .unwrap_or_else(|| unresolved("<unresolved>")),
            Expr::Field(field) => self.field_value_type(field),
            Expr::Unary(unary) if matches!(unary.op, UnOp::Deref(_)) => {
                self.field_receiver(&unary.expr)
            }
            Expr::Cast(cast) => self.resolve_type(&cast.ty),
            _ => unresolved("<unresolved>"),
        }
    }

    fn field_value_type(&self, field: &ExprField) -> TypeIdentity {
        let Member::Named(member) = &field.member else {
            return unresolved("<unresolved>");
        };
        let receiver = self.field_receiver(&field.base);
        let field_type = self
            .local_types
            .iter()
            .rev()
            .flat_map(|scope| scope.values())
            .find(|local| local.identity == receiver.name)
            .and_then(|local| local.fields.get(&member.to_string()))
            .cloned();
        let Some(field_type) = field_type else {
            return unresolved("<unresolved>");
        };
        let mut identity = self.resolve_type(&field_type);
        identity.quality = identity.quality.max(receiver.quality);
        identity
    }
}
