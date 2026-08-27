//! Field groups preserve cfg partitions while updates subtract explicit members.

use std::collections::BTreeMap;

use zrail_core::AnalysisQuality;

use crate::source::{
    CfgPredicate, SyntaxGuard, operation_model::StructUpdateFact,
    operation_place_canonical::catalog::NamedField,
};

pub(super) struct FieldGroup {
    pub(super) quality: AnalysisQuality,
    pub(super) fields: BTreeMap<String, NamedField>,
}

impl Default for FieldGroup {
    fn default() -> Self {
        Self {
            quality: AnalysisQuality::Exact,
            fields: BTreeMap::new(),
        }
    }
}

impl FieldGroup {
    pub(super) fn add(&mut self, field: NamedField) {
        self.fields
            .entry(field.name.clone())
            .and_modify(|current| {
                current.guard = SyntaxGuard::from_predicate(CfgPredicate::any(vec![
                    current.guard.predicate(),
                    field.guard.predicate(),
                ]));
                current.quality = current.quality.max(field.quality);
            })
            .or_insert(field);
    }
}

pub(super) fn omitted_guard(
    update_guard: &SyntaxGuard,
    field_guard: &SyntaxGuard,
    name: &str,
    update: &StructUpdateFact,
) -> SyntaxGuard {
    let explicit = update
        .explicit_fields
        .iter()
        .filter(|field| field.name == name)
        .map(|field| field.guard.predicate())
        .collect();
    update_guard
        .combine(field_guard)
        .combine(SyntaxGuard::from_predicate(CfgPredicate::not(
            CfgPredicate::any(explicit),
        )))
}
