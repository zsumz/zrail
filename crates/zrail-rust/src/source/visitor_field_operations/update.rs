//! Functional updates retain exact omitted-field reads or one deferred receipt.

use syn::{ExprStruct, Member, Path, spanned::Spanned};
use zrail_core::AnalysisQuality;

use super::{FactVisitor, candidates};
use crate::source::{
    CfgPredicate, SourceOperationKind, SyntaxGuard,
    attributes::cfg_guard,
    fact::source_span,
    operation_model::{
        ConstructorForm, FieldPlaceFact, StructUpdateFact, StructUpdateField, path_text,
    },
};

impl FactVisitor<'_> {
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
        let explicit_fields = expression
            .fields
            .iter()
            .filter_map(|field| match &field.member {
                Member::Named(member) => Some(StructUpdateField {
                    name: member.to_string(),
                    guard: cfg_guard(&field.attrs),
                }),
                Member::Unnamed(_) => None,
            })
            .collect::<Vec<_>>();
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
                let explicit = explicit_fields
                    .iter()
                    .filter(|candidate| candidate.name == name)
                    .map(|candidate| candidate.guard.predicate())
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
            let place = FieldPlaceFact {
                base_name: identity.name.trim_end_matches("::*").into(),
                base_quality: AnalysisQuality::Unresolved,
                base_file_local: identity.file_local,
                base_span: identity.span,
                fields: Vec::new(),
            };
            self.push_deferred_struct_update(
                &identity,
                place,
                StructUpdateFact {
                    written: path_text(&expression.path),
                    rest_span: source_span(rest.span()),
                    explicit_fields,
                },
                rest.span(),
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
}
