//! Named field syntax always emits an operation; exact identity requires a proven declaration.

use syn::{Expr, ExprField, ExprMethodCall, ExprStruct, Member, Path, spanned::Spanned};
use zrail_core::AnalysisQuality;

use crate::source::CfgPredicate;

use super::{
    FactVisitor, SourceOperationKind, SyntaxGuard,
    attributes::cfg_guard,
    fact::source_span,
    operation_model::{ConstructorForm, FieldPlaceFact, unwrapped},
    place_expression::PlaceExpression,
    visitor_patterns::PatternFieldAccess,
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
        access: PatternFieldAccess,
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
        match access {
            PatternFieldAccess::Read => self.push_field_operation(
                SourceOperationKind::FieldRead,
                &identity,
                member.to_string(),
                Some(member.span()),
                Some(place),
                guard,
            ),
            PatternFieldAccess::MutableBorrow => self.push_field_operation(
                SourceOperationKind::FieldMutableBorrow,
                &identity,
                member.to_string(),
                Some(member.span()),
                Some(place),
                guard,
            ),
            PatternFieldAccess::PossiblyMutableBorrow => {
                self.push_field_operation(
                    SourceOperationKind::FieldRead,
                    &identity,
                    member.to_string(),
                    Some(member.span()),
                    Some(place),
                    guard,
                );
                let mut unresolved = identity;
                unresolved.quality = AnalysisQuality::Unresolved;
                self.push_field_operation(
                    SourceOperationKind::FieldMutableBorrow,
                    &unresolved,
                    member.to_string(),
                    Some(member.span()),
                    None,
                    guard,
                );
            }
        }
    }

    pub(in crate::source) fn record_assignee_source_field(&mut self, path: &Path, member: &Member) {
        let Member::Named(member) = member else {
            return;
        };
        let guard = self.syntax_guard();
        self.push_path_field_read(path, &member.to_string(), member.span(), &guard);
    }

    pub(in crate::source) fn record_struct_update_reads(&mut self, expression: &ExprStruct) {
        let Some(rest) = &expression.rest else {
            return;
        };
        let base = self.resolve_identity(&expression.path);
        let fields = self
            .local_types
            .iter()
            .rev()
            .flat_map(|scope| scope.values())
            .find(|local| local.identity == base.name && local.form == ConstructorForm::Named)
            .map(|local| {
                local
                    .fields
                    .iter()
                    .map(|(name, field)| (name.clone(), field.clone()))
                    .collect::<Vec<_>>()
            });
        let guard = self.syntax_guard();
        if let Some(fields) = fields {
            for (name, field) in fields {
                let explicit = expression
                    .fields
                    .iter()
                    .filter(|candidate| match &candidate.member {
                        Member::Named(member) => member == name.as_str(),
                        Member::Unnamed(_) => false,
                    })
                    .map(|candidate| cfg_guard(&candidate.attrs).predicate())
                    .collect::<Vec<_>>();
                let omitted =
                    SyntaxGuard::from_predicate(CfgPredicate::not(CfgPredicate::any(explicit)));
                let field_guard = guard.combine(&field.guard).combine(omitted);
                if field_guard.predicate().is_satisfiable() != Some(false) {
                    self.push_path_field_read(&expression.path, &name, rest.span(), &field_guard);
                }
            }
        } else {
            let mut identity = base;
            identity.name.push_str("::*");
            identity.quality = AnalysisQuality::Unresolved;
            self.push_field_operation(
                SourceOperationKind::FieldRead,
                &identity,
                "*".into(),
                Some(rest.span()),
                None,
                &guard,
            );
        }
    }

    fn push_path_field_read(
        &mut self,
        path: &Path,
        member: &str,
        span: proc_macro2::Span,
        guard: &SyntaxGuard,
    ) {
        let base = self.resolve_identity(path);
        let identity = candidates::declared_field_identity(self, &base, member);
        let place = FieldPlaceFact {
            base_name: base.name,
            base_quality: base.quality,
            base_file_local: base.file_local,
            base_span: base.span,
            fields: vec![member.into()],
        };
        self.push_field_operation(
            SourceOperationKind::FieldRead,
            &identity,
            member.into(),
            Some(span),
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
