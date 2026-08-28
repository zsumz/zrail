//! Transactional projection replaces stale physical candidates with typed identities.

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::AnalysisQuality;

use super::{
    ObservedFact, SyntaxGuard,
    include_bindings::IncludeBindings,
    include_projection_budget::{ProjectionBudget, ProjectionLimit},
    include_projection_candidates::{ResolutionCache, aggregate},
    include_resolution_state::ResolutionUsage,
};

#[path = "include_binding_projection_retention.rs"]
mod retention;

const MAX_PROJECTED_IDENTITIES: usize = 64;

pub(super) type FactKey = (String, Option<zrail_core::SourceSpan>, SyntaxGuard);
pub(super) type CallSite = (Option<zrail_core::SourceSpan>, String, SyntaxGuard);

pub(super) struct FactProjection {
    pub(super) additions: Vec<ObservedFact>,
    pub(super) qualities: BTreeMap<FactKey, AnalysisQuality>,
    pub(super) associated_candidates: BTreeMap<FactKey, Vec<super::GenericAssociatedCandidate>>,
    pub(super) removals: BTreeSet<FactKey>,
}

pub(super) struct ProjectionRequest<'a> {
    pub(super) bindings: &'a IncludeBindings,
    pub(super) file: &'a str,
    pub(super) syntax: super::SourceSyntax,
    pub(super) facts: &'a [ObservedFact],
    pub(super) usage: ResolutionUsage,
    pub(super) call_sites: &'a BTreeSet<CallSite>,
    pub(super) project_expression: bool,
}

pub(super) fn project(
    request: &ProjectionRequest<'_>,
    uncertain: &mut Option<zrail_core::SourceSpan>,
    budget: &mut ProjectionBudget,
    remaining_file_facts: &mut usize,
    cache: &mut ResolutionCache,
) -> Result<FactProjection, ProjectionLimit> {
    let existing = request
        .facts
        .iter()
        .map(|fact| {
            (
                (fact.name.clone(), fact.span, fact.guard.clone()),
                fact.quality,
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut additions = BTreeMap::<FactKey, ObservedFact>::new();
    let mut qualities = BTreeMap::<FactKey, AnalysisQuality>::new();
    let mut associated_candidates = BTreeMap::new();
    let mut removals = BTreeSet::new();
    for fact in request.facts.iter().filter(|fact| fact.written.is_some()) {
        let usage = retention::fact_usage(fact, request.usage, request.call_sites);
        let instances =
            request
                .bindings
                .active_instances(request.file, request.syntax, &fact.guard);
        if instances.is_empty() {
            continue;
        }
        let (aggregate, compatible, test_coverage) =
            aggregate(request.bindings, fact, &instances, usage, budget, cache)?;
        if aggregate.len() > MAX_PROJECTED_IDENTITIES {
            *uncertain = uncertain.or(fact.span);
            continue;
        }
        let authoritative = compatible
            && aggregate.len() == 1
            && aggregate.values().all(|candidate| {
                candidate.instances == instances.len()
                    && (candidate.quality != AnalysisQuality::Unresolved
                        || candidate.generic_shadow.is_some())
            });
        if authoritative {
            removals.extend(
                request
                    .facts
                    .iter()
                    .filter(|stale| {
                        stale.span == fact.span
                            && stale.guard == fact.guard
                            && !aggregate.contains_key(&stale.name)
                    })
                    .map(|stale| (stale.name.clone(), stale.span, stale.guard.clone())),
            );
        }
        let mut state = retention::RetentionState {
            project_expression: request.project_expression,
            existing: &existing,
            additions: &mut additions,
            qualities: &mut qualities,
            associated_candidates: &mut associated_candidates,
            uncertain,
            budget,
            remaining_file_facts,
        };
        retention::retain_candidates(
            fact,
            aggregate,
            compatible,
            instances.len(),
            test_coverage,
            &mut state,
        )?;
    }
    Ok(FactProjection {
        additions: additions.into_values().collect(),
        qualities,
        associated_candidates,
        removals,
    })
}
