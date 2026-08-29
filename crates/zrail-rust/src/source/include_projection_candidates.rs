//! Candidate aggregation preserves per-domain compatibility and quality.

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::AnalysisQuality;

use super::{
    GenericRootShadow, ImplicitPreludeEligibility, ObservedFact, SourceInstanceId, SyntaxGuard,
    include_binding_helpers::{lexical_shadow, normalize, unresolved},
    include_bindings::{IncludeBindings, ResolvedOrigin, ResolvedPath, ResolvedTerminal},
    include_projection_budget::{ProjectionBudget, ProjectionLimit},
    include_resolution_state::{ResolutionUsage, WrittenResolveRequest},
};

#[path = "include_projection_aggregate.rs"]
mod aggregate_model;
#[path = "include_projection_context.rs"]
mod context;
pub(super) use aggregate_model::{CandidateAggregate, TestCoverage};

pub(super) type ResolutionCache = BTreeMap<ResolutionCacheKey, Vec<ResolvedPath>>;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct ResolutionCacheKey {
    instance: SourceInstanceId,
    written: String,
    scope: Vec<zrail_core::SourceSpan>,
    usage: ResolutionUsage,
    guard: SyntaxGuard,
    implicit_prelude: ImplicitPreludeEligibility,
    generic_shadow: Option<GenericRootShadow>,
    inherits_parent_context: bool,
}

pub(super) fn aggregate(
    bindings: &IncludeBindings,
    fact: &ObservedFact,
    instances: &[SourceInstanceId],
    usage: ResolutionUsage,
    budget: &mut ProjectionBudget,
    cache: &mut ResolutionCache,
) -> Result<(BTreeMap<String, CandidateAggregate>, bool, TestCoverage), ProjectionLimit> {
    let mut aggregate = BTreeMap::<String, CandidateAggregate>::new();
    let mut compatible = true;
    let mut test_coverage = TestCoverage::default();
    let mut common = None;
    for instance in instances {
        let written = fact.written.as_deref().unwrap_or(&fact.name);
        let source = bindings.instances.get(*instance);
        let generic_identity = shadow::generic_identity(fact, usage, source);
        let implicit_prelude = source.map_or(fact.implicit_prelude, |source| {
            shadow::eligibility(fact, usage, source)
        });
        let key = ResolutionCacheKey {
            instance: *instance,
            written: written.into(),
            scope: fact.lexical_scope.clone(),
            usage,
            guard: fact.guard.clone(),
            implicit_prelude,
            generic_shadow: generic_identity.as_ref().map(|identity| identity.shadow),
            inherits_parent_context: fact.inherits_parent_context,
        };
        let resolved = if let Some(resolved) = cache.get(&key) {
            resolved.clone()
        } else {
            let contextual_self = match source {
                Some(source) => {
                    context::current_self(bindings, fact, *instance, source, usage, budget)?
                }
                None => None,
            };
            let resolved = if let Some(resolved) = contextual_self {
                resolved
            } else if let Some(identity) = &generic_identity {
                vec![ResolvedPath {
                    name: identity.name.clone(),
                    quality: identity.quality,
                    crossed_include: source.is_some_and(|source| source.parent.is_some()),
                    requires_projection: true,
                    blocks_completeness: false,
                    origin: ResolvedOrigin::CrateLocal,
                    terminal: if identity.is_associated() {
                        ResolvedTerminal::Unknown
                    } else {
                        match identity.shadow {
                            GenericRootShadow::TypeParameter => ResolvedTerminal::Type,
                            GenericRootShadow::ConstParameter => ResolvedTerminal::Value,
                        }
                    },
                }]
            } else {
                match implicit_prelude {
                    ImplicitPreludeEligibility::LocalShadow => vec![lexical_shadow(written, usage)],
                    ImplicitPreludeEligibility::GenericShadow => {
                        vec![unresolved(written)]
                    }
                    ImplicitPreludeEligibility::PossibleShadow => {
                        let mut candidates = bindings.resolve_written(
                            &WrittenResolveRequest {
                                instance: *instance,
                                written,
                                scope: &fact.lexical_scope,
                                depth: 0,
                                usage,
                                guard: &fact.guard,
                                allow_implicit_prelude: true,
                            },
                            &mut BTreeSet::new(),
                            budget,
                        )?;
                        for candidate in &mut candidates {
                            candidate.quality = AnalysisQuality::Unresolved;
                            candidate.blocks_completeness = true;
                        }
                        candidates.push(unresolved(written));
                        normalize(candidates)
                    }
                    ImplicitPreludeEligibility::Eligible | ImplicitPreludeEligibility::Disabled => {
                        let mut seen = BTreeSet::new();
                        bindings.resolve_written(
                            &WrittenResolveRequest {
                                instance: *instance,
                                written,
                                scope: &fact.lexical_scope,
                                depth: 0,
                                usage,
                                guard: &fact.guard,
                                allow_implicit_prelude: implicit_prelude
                                    == ImplicitPreludeEligibility::Eligible,
                            },
                            &mut seen,
                            budget,
                        )?
                    }
                }
            };
            cache.insert(key, resolved.clone());
            resolved
        };
        let test_instance = source.is_some_and(|source| source.domain.mode.enables_cfg_test());
        let associated_candidates = if !fact.associated_candidates.is_empty()
            || generic_identity
                .as_ref()
                .is_some_and(super::GenericRootIdentity::is_associated)
        {
            match source {
                Some(source) => {
                    context::associated_candidates(bindings, fact, *instance, source, budget)?
                }
                None => fact.associated_candidates.clone(),
            }
        } else {
            Vec::new()
        };
        if test_instance {
            test_coverage.instances += 1;
            test_coverage.compatible &= resolved.len() == 1;
        }
        compatible &= resolved.len() == 1;
        let names = resolved
            .iter()
            .map(|candidate| candidate.name.as_str())
            .collect::<BTreeSet<_>>();
        let only = (names.len() == 1)
            .then(|| names.iter().next().copied())
            .flatten();
        common = match (common, only) {
            (None, name) => name.map(str::to_owned),
            (Some(current), Some(name)) if current == name => Some(current),
            _ => {
                compatible = false;
                None
            }
        };
        for candidate in resolved {
            let generic_candidate = generic_identity
                .as_ref()
                .is_some_and(|identity| candidate.name == identity.name);
            let entry = aggregate.entry(candidate.name).or_default();
            entry.instances += 1;
            entry.test_instances += usize::from(test_instance);
            entry.quality = entry.quality.max(candidate.quality);
            entry.requires_projection |= candidate.requires_projection;
            entry.blocks_completeness |= candidate.blocks_completeness;
            if generic_candidate {
                entry.generic_shadow = generic_identity.as_ref().map(|identity| identity.shadow);
            }
            if generic_candidate || !associated_candidates.is_empty() {
                for candidate in &associated_candidates {
                    let key = (
                        candidate.name.clone(),
                        candidate.projection.clone(),
                        candidate.kind,
                    );
                    entry
                        .associated_candidates
                        .entry(key)
                        .and_modify(|existing| {
                            existing.quality = existing.quality.max(candidate.quality);
                            existing.provider_complete &= candidate.provider_complete;
                            existing
                                .provider_authorities
                                .extend(candidate.provider_authorities.iter().cloned());
                        })
                        .or_insert_with(|| candidate.clone());
                }
            }
            entry.production |= bindings
                .instances
                .get(*instance)
                .is_some_and(|source| !source.domain.mode.enables_cfg_test());
        }
    }
    Ok((aggregate, compatible, test_coverage))
}

#[path = "include_projection_shadow.rs"]
mod shadow;
