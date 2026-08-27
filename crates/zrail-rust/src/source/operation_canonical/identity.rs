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
    associated: &super::associated::Catalog,
    budget: &mut ProjectionBudget,
    unresolved: &mut BTreeSet<(String, Option<SourceSpan>)>,
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
        for route in &mut result.routes {
            associated.classify_value(route, &operation.identity.guard);
        }
        if result.expected == 0 {
            canonical.push(operation);
            continue;
        }
        let mut candidates = BTreeMap::<String, Candidate>::new();
        let mut removed = false;
        let mut unknown = false;
        for route in result.routes {
            match disposition(operation.kind, construction, route.terminal) {
                Disposition::Discard => removed = true,
                Disposition::Keep(form) => insert(&mut candidates, route, form),
                Disposition::Unknown => {
                    unknown = true;
                    insert(&mut candidates, route, ConstructorForm::Unknown);
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
        let retained_form = retained_form(candidates.values().map(|candidate| candidate.form));
        if operation.kind == SourceOperationKind::ConstructorCapability {
            operation.kind = if retained_form == ConstructorForm::Unit {
                SourceOperationKind::TypeConstruction
            } else {
                SourceOperationKind::ConstructorCapability
            };
            operation.construction = Some(retained_form);
            operation.construction_proven = !unknown && retained_form != ConstructorForm::Unknown;
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
    form: ConstructorForm,
}

fn insert(
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

fn retained_form(forms: impl Iterator<Item = ConstructorForm>) -> ConstructorForm {
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
