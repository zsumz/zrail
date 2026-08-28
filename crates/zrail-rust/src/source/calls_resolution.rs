//! Resolution boundaries retain only policy-relevant associated candidates.

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::{AnalysisQuality, Finding};

use super::super::{
    AssociatedOccurrenceKind, CompilationDomain, FactNamespace, GenericAssociatedCandidate,
    GenericRootShadow, ObservedFact, RustFileFacts, SyntaxGuard,
    model::{CallResolutionFact, CallResolutionKind},
};

pub(in crate::source) fn resolution_findings(
    path: &str,
    calls: &[CallResolutionFact],
    domains: Option<&BTreeSet<CompilationDomain>>,
) -> Vec<Finding> {
    let Some(domains) = domains else {
        return Vec::new();
    };
    calls
        .iter()
        .filter(|call| {
            domains.iter().any(|domain| {
                call.guard
                    .available_in(SyntaxGuard::for_test_only(domain.mode.enables_cfg_test()))
            })
        })
        .map(|call| resolution_finding(path, call))
        .collect()
}

pub(in crate::source) fn normalize_resolutions(boundaries: &mut Vec<CallResolutionFact>) {
    let mut normalized = Vec::<CallResolutionFact>::new();
    for boundary in std::mem::take(boundaries) {
        let existing = normalized.iter_mut().find(|existing| {
            existing.written == boundary.written
                && existing.span == boundary.span
                && existing.guard == boundary.guard
                && existing.kind == boundary.kind
        });
        if let Some(existing) = existing {
            merge_candidates(
                &mut existing.associated_candidates,
                &boundary.associated_candidates,
            );
            if existing.occurrence != Some(AssociatedOccurrenceKind::DirectCall) {
                existing.occurrence = boundary.occurrence.or(existing.occurrence);
            }
        } else {
            normalized.push(boundary);
        }
    }
    *boundaries = normalized;
}

pub(crate) fn resolution_finding(path: &str, call: &CallResolutionFact) -> Finding {
    let (message, help) = match call.kind {
        CallResolutionKind::AssociatedTypeProjection
        | CallResolutionKind::ContextualAssociatedTypeProjection => (
            format!(
                "qualified expression path crosses an associated-type projection that zrail cannot resolve exactly: {}",
                call.written
            ),
            "name one concrete type before trusting path or direct-call authority at this site",
        ),
        CallResolutionKind::ExplicitTrait => (
            format!(
                "explicit trait in qualified associated-item path cannot be resolved exactly: {}",
                call.written
            ),
            "import or qualify the exact trait before trusting associated-item or direct-call authority at this site",
        ),
        CallResolutionKind::GenericAssociatedItem => (
            format!(
                "generic-root associated item cannot be resolved to one exact policy identity: {}",
                call.written
            ),
            "use an explicit trait-qualified path before trusting associated-item, capability, or direct-call authority at this site",
        ),
    };
    Finding::error(
        "RUST-CALL-001",
        "rust.source.call-resolution",
        "source",
        message,
    )
    .at(path, Some(call.span))
    .with_analysis(AnalysisQuality::Unresolved)
    .with_help(help)
}

pub(in crate::source) fn generic_resolution_boundaries(
    file: &RustFileFacts,
) -> Vec<CallResolutionFact> {
    let call_sites = file
        .calls
        .iter()
        .filter_map(boundary_key)
        .collect::<BTreeSet<_>>();
    let mut boundaries = BTreeMap::<
        (String, zrail_core::SourceSpan, SyntaxGuard),
        (AssociatedOccurrenceKind, Vec<GenericAssociatedCandidate>),
    >::new();
    for fact in file.paths.iter().chain(&file.calls).filter(|fact| {
        (fact.generic_shadow == Some(GenericRootShadow::TypeParameter)
            || !fact.associated_candidates.is_empty())
            && fact
                .written
                .as_deref()
                .is_some_and(|written| written.contains("::"))
    }) {
        let (Some(written), Some(span)) = (fact.written.as_ref(), fact.span) else {
            continue;
        };
        let key = (written.clone(), span, fact.guard.clone());
        let occurrence = if call_sites.contains(&key) {
            AssociatedOccurrenceKind::DirectCall
        } else if fact.namespace == FactNamespace::Type {
            AssociatedOccurrenceKind::TypeReference
        } else {
            AssociatedOccurrenceKind::ValueReference
        };
        let entry = boundaries
            .entry(key)
            .or_insert_with(|| (occurrence, Vec::new()));
        if occurrence == AssociatedOccurrenceKind::DirectCall {
            entry.0 = occurrence;
        }
        merge_candidates(&mut entry.1, &fact.associated_candidates);
    }
    boundaries
        .into_iter()
        .map(
            |((written, span, guard), (occurrence, associated_candidates))| CallResolutionFact {
                written,
                span,
                guard,
                kind: CallResolutionKind::GenericAssociatedItem,
                associated_candidates,
                occurrence: Some(occurrence),
            },
        )
        .collect()
}

fn boundary_key(fact: &ObservedFact) -> Option<(String, zrail_core::SourceSpan, SyntaxGuard)> {
    (fact.generic_shadow == Some(GenericRootShadow::TypeParameter)
        || !fact.associated_candidates.is_empty())
    .then(|| Some((fact.written.clone()?, fact.span?, fact.guard.clone())))
    .flatten()
}

fn merge_candidates(
    target: &mut Vec<GenericAssociatedCandidate>,
    candidates: &[GenericAssociatedCandidate],
) {
    for candidate in candidates {
        if let Some(existing) = target.iter_mut().find(|existing| {
            existing.name == candidate.name && existing.canonical == candidate.canonical
        }) {
            existing.quality = existing.quality.max(candidate.quality);
        } else {
            target.push(candidate.clone());
        }
    }
    target.sort();
}
