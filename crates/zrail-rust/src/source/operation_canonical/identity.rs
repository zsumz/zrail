//! Construction identities retain canonical candidates from every source instance.

#[path = "identity/candidates.rs"]
mod candidates;

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::{AnalysisQuality, SourceSpan};

use super::{
    super::{ConstructorForm, SourceOperationFact, SourceOperationKind},
    resolution,
};
use crate::source::{
    CallResolutionFact, CallResolutionKind,
    include_bindings::{IncludeBindings, ResolvedTerminal},
    include_projection_budget::{ProjectionBudget, ProjectionLimit},
    include_resolution_state::ResolutionUsage,
};

pub(super) fn canonicalize(
    operations: &mut Vec<SourceOperationFact>,
    bindings: &IncludeBindings,
    file: &str,
    associated: &super::associated::Catalog,
    budget: &mut ProjectionBudget,
    unresolved: &mut BTreeSet<(String, Option<SourceSpan>)>,
    call_resolutions: &mut Vec<CallResolutionFact>,
) -> Result<(), ProjectionLimit> {
    let mut canonical = Vec::with_capacity(operations.len());
    for mut operation in operations.drain(..) {
        if !matches!(
            operation.kind,
            SourceOperationKind::TypeConstruction | SourceOperationKind::ConstructorCapability
        ) {
            canonical.push(operation);
            continue;
        }
        let Some(construction) = operation.construction else {
            canonical.push(operation);
            continue;
        };
        let written = operation.qualified_subject.as_ref().map_or_else(
            || {
                operation
                    .identity
                    .written
                    .as_deref()
                    .unwrap_or(&operation.identity.name)
            },
            |subject| subject.lookup.as_str(),
        );
        let usage = if construction == ConstructorForm::Named {
            ResolutionUsage::OperationType
        } else {
            ResolutionUsage::ConstructorValue
        };
        let mut result = resolution::resolve(
            resolution::Request {
                bindings,
                file,
                fact: &operation.identity,
                file_local: operation.file_local
                    && operation.subject_origin
                        != super::super::operation_model::OperationSubjectOrigin::LocalDeclaration
                    && (construction == ConstructorForm::Named || operation.construction_proven),
                subject_origin: operation.subject_origin,
                written,
                usage,
                construction: Some(construction),
            },
            budget,
        )?;
        let qualification = super::qualification::classify(
            operation.qualified_subject.as_ref(),
            &mut result,
            associated,
            &operation.identity.guard,
            operation
                .identity
                .written
                .as_deref()
                .unwrap_or(&operation.identity.name),
            bindings,
            file,
            budget,
        )?;
        if let super::qualification::Disposition::AssociatedItem(quality) = qualification {
            if quality != AnalysisQuality::Exact
                && let Some(span) = operation.identity.span
            {
                let boundary = CallResolutionFact {
                    written: operation
                        .identity
                        .written
                        .clone()
                        .unwrap_or_else(|| operation.identity.name.clone()),
                    span,
                    guard: operation.identity.guard.clone(),
                    kind: CallResolutionKind::ExplicitTrait,
                };
                if !call_resolutions.contains(&boundary) {
                    call_resolutions.push(boundary);
                }
            }
            continue;
        }
        if result.expected == 0 {
            canonical.push(operation);
            continue;
        }
        let mut candidates = BTreeMap::<String, candidates::Candidate>::new();
        let mut removed = false;
        let mut discarded_exact = true;
        let mut unknown = false;
        let route_count = result.routes.len();
        for route in result.routes {
            match disposition(operation.kind, construction, route.terminal) {
                Disposition::Discard => {
                    removed = true;
                    discarded_exact &= route.quality == AnalysisQuality::Exact;
                }
                Disposition::Keep(form) => candidates::insert(&mut candidates, route, form),
                Disposition::Unknown => {
                    unknown = true;
                    candidates::insert(&mut candidates, route, ConstructorForm::Unknown);
                }
            }
        }
        if candidates.is_empty() {
            if !discarded_exact || route_count < result.expected {
                operation.identity.quality = AnalysisQuality::Unresolved;
                operation.identity.canonical.clear();
                canonical.push(operation);
            }
            continue;
        }
        let mut quality = candidates
            .values()
            .map(|candidate| candidate.quality)
            .max()
            .unwrap_or(AnalysisQuality::Unresolved);
        if result.unresolved || unknown {
            quality = AnalysisQuality::Unresolved;
        } else if candidates.len() > 1 || removed {
            quality = quality.max(AnalysisQuality::Conservative);
        }
        let retained_form =
            candidates::retained_form(candidates.values().map(|candidate| candidate.form));
        if operation.kind == SourceOperationKind::ConstructorCapability {
            operation.kind = if retained_form == ConstructorForm::Unit {
                SourceOperationKind::TypeConstruction
            } else {
                SourceOperationKind::ConstructorCapability
            };
            operation.construction = Some(retained_form);
            operation.construction_proven = !unknown && retained_form != ConstructorForm::Unknown;
        }
        candidates::apply(&mut operation, candidates, quality);
        operation.file_local = false;
        if construction == ConstructorForm::Named && result.blocks_completeness {
            unresolved.insert((file.into(), operation.identity.span));
        }
        canonical.push(operation);
    }
    *operations = canonical;
    Ok(())
}

enum Disposition {
    Keep(ConstructorForm),
    Discard,
    Unknown,
}

fn disposition(
    kind: SourceOperationKind,
    construction: ConstructorForm,
    terminal: ResolvedTerminal,
) -> Disposition {
    if kind == SourceOperationKind::ConstructorCapability {
        return match terminal {
            ResolvedTerminal::Constructor(ConstructorForm::Tuple) => {
                Disposition::Keep(ConstructorForm::Tuple)
            }
            ResolvedTerminal::Constructor(ConstructorForm::Unit) => {
                Disposition::Keep(ConstructorForm::Unit)
            }
            ResolvedTerminal::Constructor(ConstructorForm::Unknown) | ResolvedTerminal::Unknown => {
                Disposition::Unknown
            }
            ResolvedTerminal::Constructor(ConstructorForm::Named)
            | ResolvedTerminal::Type
            | ResolvedTerminal::Value
            | ResolvedTerminal::Module => Disposition::Discard,
        };
    }
    if matches!(construction, ConstructorForm::Named) {
        return Disposition::Keep(ConstructorForm::Named);
    }
    match terminal {
        ResolvedTerminal::Constructor(form) if form == construction => {
            Disposition::Keep(construction)
        }
        ResolvedTerminal::Constructor(ConstructorForm::Unknown) | ResolvedTerminal::Unknown => {
            Disposition::Unknown
        }
        ResolvedTerminal::Constructor(_)
        | ResolvedTerminal::Type
        | ResolvedTerminal::Value
        | ResolvedTerminal::Module => Disposition::Discard,
    }
}
