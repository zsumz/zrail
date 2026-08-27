//! One bounded adapter exposes guarded type identity to every operation consumer.

use zrail_core::AnalysisQuality;

use super::super::{
    CompilationDomain, ConstructorForm, GuardAvailability, ObservedFact,
    include_binding_helpers::canonical_local_name,
    include_bindings::{IncludeBindings, ResolvedOrigin, ResolvedTerminal},
    include_projection_budget::{ProjectionBudget, ProjectionLimit},
    include_resolution_state::{ResolutionTrail, ResolutionUsage, WrittenResolveRequest},
    operation_model::OperationSubjectOrigin,
};

#[derive(Clone)]
pub(super) struct Route {
    pub(super) domain: CompilationDomain,
    pub(super) name: String,
    pub(super) quality: AnalysisQuality,
    pub(super) origin: ResolvedOrigin,
    pub(super) terminal: ResolvedTerminal,
}

#[derive(Clone)]
pub(super) struct Resolution {
    pub(super) expected: usize,
    pub(super) routes: Vec<Route>,
    pub(super) unresolved: bool,
    pub(super) blocks_completeness: bool,
}

#[derive(Clone, Copy)]
pub(super) struct Request<'a> {
    pub(super) bindings: &'a IncludeBindings,
    pub(super) file: &'a str,
    pub(super) fact: &'a ObservedFact,
    pub(super) file_local: bool,
    pub(super) subject_origin: OperationSubjectOrigin,
    pub(super) written: &'a str,
    pub(super) usage: ResolutionUsage,
    pub(super) construction: Option<ConstructorForm>,
}

pub(super) fn resolve(
    request: Request<'_>,
    budget: &mut ProjectionBudget,
) -> Result<Resolution, ProjectionLimit> {
    let Request {
        bindings,
        file,
        fact,
        file_local,
        subject_origin,
        written,
        usage,
        construction,
    } = request;
    let instances = bindings
        .instances
        .for_file(file)
        .iter()
        .copied()
        .filter(|id| {
            bindings.instances.get(*id).is_some_and(|source| {
                source
                    .guard
                    .combine(&fact.guard)
                    .availability_in_domain(&source.domain)
                    .is_available()
            })
        })
        .collect::<Vec<_>>();
    let mut resolution = Resolution {
        expected: instances.len(),
        routes: Vec::new(),
        unresolved: false,
        blocks_completeness: false,
    };
    for instance in instances {
        let Some(source) = bindings.instances.get(instance) else {
            resolution.unresolved = true;
            resolution.blocks_completeness = true;
            continue;
        };
        let guard_quality = match source
            .guard
            .combine(&fact.guard)
            .availability_in_domain(&source.domain)
        {
            GuardAvailability::Exact => AnalysisQuality::Exact,
            GuardAvailability::Possible => AnalysisQuality::Conservative,
            GuardAvailability::Absent => continue,
        };
        if file_local {
            let Some(module) = bindings.effective_module(instance, &[], budget)? else {
                resolution.unresolved = true;
                resolution.blocks_completeness = true;
                continue;
            };
            let Some(name) = canonical_local_name(&module.names, &fact.name) else {
                resolution.unresolved = true;
                resolution.blocks_completeness = true;
                continue;
            };
            resolution.routes.push(Route {
                domain: source.domain.clone(),
                name,
                quality: fact.quality.max(guard_quality),
                origin: ResolvedOrigin::CrateLocal,
                terminal: construction.map_or(ResolvedTerminal::Type, |form| {
                    ResolvedTerminal::Constructor(form)
                }),
            });
            continue;
        }
        let lookup = if subject_origin == OperationSubjectOrigin::CurrentSelf {
            &fact.name
        } else {
            written
        };
        let mut trail = ResolutionTrail::new();
        let resolved = bindings.resolve_written(
            &WrittenResolveRequest {
                instance,
                written: lookup,
                scope: &fact.lexical_scope,
                depth: 0,
                usage,
                guard: &fact.guard,
            },
            &mut trail,
            budget,
        )?;
        let ambiguous = resolved.len() != 1;
        resolution.unresolved |= ambiguous || resolved.is_empty();
        for candidate in resolved {
            let quality = candidate.quality.max(guard_quality).max(if ambiguous {
                AnalysisQuality::Unresolved
            } else {
                AnalysisQuality::Exact
            });
            resolution.unresolved |= quality == AnalysisQuality::Unresolved;
            resolution.blocks_completeness |= candidate.blocks_completeness;
            resolution.routes.push(Route {
                domain: source.domain.clone(),
                name: candidate.name,
                quality,
                origin: candidate.origin,
                terminal: candidate.terminal,
            });
        }
    }
    if resolution.expected > 0 && resolution.routes.len() < resolution.expected {
        resolution.unresolved = true;
        resolution.blocks_completeness = true;
    }
    Ok(resolution)
}
