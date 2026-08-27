//! Construction identities retain canonical candidates from every source instance.

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::{AnalysisQuality, SourceSpan};

use super::{
    super::{ConstructorForm, SourceOperationFact, SourceOperationKind},
    resolution,
};
use crate::source::{
    include_bindings::{IncludeBindings, ResolvedOrigin, ResolvedTerminal},
    include_projection_budget::{ProjectionBudget, ProjectionLimit},
    include_resolution_state::ResolutionUsage,
};

pub(super) fn canonicalize(
    operations: &mut Vec<SourceOperationFact>,
    bindings: &IncludeBindings,
    file: &str,
    budget: &mut ProjectionBudget,
    unresolved: &mut BTreeSet<(String, Option<SourceSpan>)>,
) -> Result<(), ProjectionLimit> {
    let mut canonical = Vec::with_capacity(operations.len());
    for mut operation in operations.drain(..) {
        if operation.kind != SourceOperationKind::TypeConstruction {
            canonical.push(operation);
            continue;
        }
        let Some(construction) = operation.construction else {
            canonical.push(operation);
            continue;
        };
        let written = operation
            .identity
            .written
            .as_deref()
            .unwrap_or(&operation.identity.name);
        let usage = if construction == ConstructorForm::Named {
            ResolutionUsage::OperationType
        } else {
            ResolutionUsage::ConstructorValue
        };
        let result = resolution::resolve(
            resolution::Request {
                bindings,
                file,
                fact: &operation.identity,
                file_local: operation.file_local
                    && (construction == ConstructorForm::Named || operation.construction_proven),
                subject_origin: operation.subject_origin,
                written,
                usage,
                construction: Some(construction),
            },
            budget,
        )?;
        if result.expected == 0 {
            canonical.push(operation);
            continue;
        }
        let mut candidates = BTreeMap::<String, Candidate>::new();
        let mut removed = false;
        let mut unknown = false;
        for route in result.routes {
            match disposition(construction, route.terminal) {
                Disposition::Discard => removed = true,
                Disposition::Keep => insert(&mut candidates, route),
                Disposition::Unknown => {
                    unknown = true;
                    insert(&mut candidates, route);
                }
            }
        }
        if candidates.is_empty() {
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
        apply_candidates(&mut operation, candidates, quality);
        operation.file_local = false;
        if construction == ConstructorForm::Named && result.blocks_completeness {
            unresolved.insert((file.into(), operation.identity.span));
        }
        canonical.push(operation);
    }
    *operations = canonical;
    Ok(())
}

#[derive(Clone)]
struct Candidate {
    quality: AnalysisQuality,
    origin: ResolvedOrigin,
}

fn insert(candidates: &mut BTreeMap<String, Candidate>, route: resolution::Route) {
    candidates
        .entry(route.name)
        .and_modify(|candidate| {
            candidate.quality = candidate.quality.max(route.quality);
            if candidate.origin != route.origin {
                candidate.origin = ResolvedOrigin::Unknown;
                candidate.quality = AnalysisQuality::Unresolved;
            }
        })
        .or_insert(Candidate {
            quality: route.quality,
            origin: route.origin,
        });
}

fn apply_candidates(
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

enum Disposition {
    Keep,
    Discard,
    Unknown,
}

fn disposition(construction: ConstructorForm, terminal: ResolvedTerminal) -> Disposition {
    if matches!(construction, ConstructorForm::Named) {
        return Disposition::Keep;
    }
    match terminal {
        ResolvedTerminal::Constructor(form) if form == construction => Disposition::Keep,
        ResolvedTerminal::Constructor(ConstructorForm::Unknown) | ResolvedTerminal::Unknown => {
            Disposition::Unknown
        }
        ResolvedTerminal::Constructor(_)
        | ResolvedTerminal::Type
        | ResolvedTerminal::Value
        | ResolvedTerminal::Module => Disposition::Discard,
    }
}
