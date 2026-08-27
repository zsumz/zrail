//! Functional updates retain exact omitted-field reads or one deferred receipt.

use syn::{ExprStruct, Member, Path, spanned::Spanned};
use zrail_core::AnalysisQuality;

use super::{FactVisitor, candidates};
use crate::source::{
    SourceOperationKind, SyntaxGuard,
    attributes::cfg_guard,
    fact::source_span,
    operation_model::{FieldPlaceFact, StructUpdateFact, StructUpdateField, path_text},
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
        // A written update path can be shadowed by block-local type and import
        // bindings. Retain it for the guarded binding graph instead of using
        // the visitor's declaration cache, which cannot represent every Rust
        // namespace shadow.
        let base = self.resolve_construction_identity(&expression.path);
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
        let guard = self.syntax_guard();
        let place = FieldPlaceFact {
            base_name: base.name.clone(),
            base_quality: base.quality,
            base_file_local: base.file_local,
            base_origin: base.origin,
            base_span: base.span,
            fields: Vec::new(),
        };
        let mut identity = base;
        identity.name.push_str("::*");
        identity.quality = AnalysisQuality::Unresolved;
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
            base_origin: base.origin,
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
