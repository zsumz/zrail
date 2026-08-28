//! One bounded adapter exposes guarded type identity to every operation consumer.

use zrail_core::AnalysisQuality;

use super::super::{
    CompilationDomain, ConstructorForm, GenericRootShadow, GuardAvailability, ObservedFact,
    RootLookupNamespace, generic_root_shadow,
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
    pub(super) suppressed: usize,
    pub(super) routes: Vec<Route>,
    pub(super) unresolved: bool,
    pub(super) blocks_completeness: bool,
}

#[derive(Clone, Copy)]
pub(super) struct Request<'a> {
    pub(super) bindings: &'a IncludeBindings,
    pub(super) file: &'a str,
    pub(super) syntax: super::super::SourceSyntax,
    pub(super) fact: &'a ObservedFact,
    pub(super) file_local: bool,
    pub(super) subject_origin: OperationSubjectOrigin,
    pub(super) written: &'a str,
    pub(super) usage: ResolutionUsage,
    pub(super) construction: Option<ConstructorForm>,
    pub(super) root_lookup: Option<RootLookupNamespace>,
    pub(super) generic_shadow: Option<GenericRootShadow>,
}

pub(super) fn resolve(
    request: Request<'_>,
    budget: &mut ProjectionBudget,
) -> Result<Resolution, ProjectionLimit> {
    let Request {
        bindings,
        file,
        syntax,
        fact,
        file_local,
        subject_origin,
        written,
        usage,
        construction,
        root_lookup,
        generic_shadow,
    } = request;
    let instances = bindings
        .instances
        .for_source(file, syntax)
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
        expected: 0,
        suppressed: 0,
        routes: Vec::new(),
        unresolved: false,
        blocks_completeness: false,
    };
    for instance in instances {
        let Some(source) = bindings.instances.get(instance) else {
            resolution.expected += 1;
            resolution.unresolved = true;
            resolution.blocks_completeness = true;
            continue;
        };
        let inherited_generic_shadow = root_lookup.and_then(|lookup| {
            generic_root_shadow(
                written,
                lookup,
                &source.generic_types,
                &source.generic_values,
            )
        });
        if generic_shadow.is_some() || inherited_generic_shadow.is_some() {
            resolution.suppressed += 1;
            continue;
        }
        let value_shadow = if root_lookup == Some(RootLookupNamespace::Value) {
            source.value_shadow_availability(written, &fact.guard)
        } else {
            GuardAvailability::Absent
        };
        if value_shadow == GuardAvailability::Exact {
            resolution.suppressed += 1;
            continue;
        }
        resolution.expected += 1;
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
                allow_implicit_prelude: true,
            },
            &mut trail,
            budget,
        )?;
        let ambiguous = resolved.len() != 1;
        resolution.unresolved |= ambiguous || resolved.is_empty();
        for candidate in resolved {
            let quality = candidate.quality.max(guard_quality).max(
                if ambiguous || value_shadow == GuardAvailability::Possible {
                    AnalysisQuality::Unresolved
                } else {
                    AnalysisQuality::Exact
                },
            );
            resolution.unresolved |= quality == AnalysisQuality::Unresolved;
            resolution.blocks_completeness |=
                candidate.blocks_completeness || value_shadow == GuardAvailability::Possible;
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
