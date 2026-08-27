//! Canonical constructor candidates merge without losing origin or form ambiguity.

use std::collections::BTreeMap;

use zrail_core::AnalysisQuality;

use super::super::resolution;
use crate::source::{ConstructorForm, SourceOperationFact, include_bindings::ResolvedOrigin};

pub(super) struct Candidate {
    pub(super) quality: AnalysisQuality,
    origin: ResolvedOrigin,
    pub(super) form: ConstructorForm,
}

pub(super) fn insert(
    candidates: &mut BTreeMap<String, Candidate>,
    route: resolution::Route,
    form: ConstructorForm,
) {
    candidates
        .entry(route.name)
        .and_modify(|candidate| {
            candidate.quality = candidate.quality.max(route.quality);
            if candidate.origin != route.origin {
                candidate.origin = ResolvedOrigin::Unknown;
                candidate.quality = AnalysisQuality::Unresolved;
            }
            if candidate.form != form {
                candidate.form = ConstructorForm::Unknown;
                candidate.quality = candidate.quality.max(AnalysisQuality::Conservative);
            }
        })
        .or_insert(Candidate {
            quality: route.quality,
            origin: route.origin,
            form,
        });
}

pub(super) fn retained_form(forms: impl Iterator<Item = ConstructorForm>) -> ConstructorForm {
    forms
        .reduce(|left, right| {
            if left == right {
                left
            } else {
                ConstructorForm::Unknown
            }
        })
        .unwrap_or(ConstructorForm::Unknown)
}

pub(super) fn apply(
    operation: &mut SourceOperationFact,
    candidates: BTreeMap<String, Candidate>,
    quality: AnalysisQuality,
) {
    if candidates.len() == 1 {
        let Some((name, candidate)) = candidates.into_iter().next() else {
            return;
        };
        operation.identity.name.clone_from(&name);
        operation.identity.canonical = if candidate.origin == ResolvedOrigin::CrateLocal {
            vec![name]
        } else {
            Vec::new()
        };
        operation.identity.quality = quality.max(if candidate.origin == ResolvedOrigin::Unknown {
            AnalysisQuality::Unresolved
        } else {
            AnalysisQuality::Exact
        });
    } else {
        operation.identity.canonical = candidates.into_keys().collect();
        operation.identity.quality = quality.max(AnalysisQuality::Conservative);
    }
}
