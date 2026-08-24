//! Transactional projection replaces stale physical candidates with typed identities.

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::AnalysisQuality;

use super::{
    ObservedFact, SyntaxGuard,
    include_bindings::IncludeBindings,
    include_projection_budget::{ProjectionBudget, ProjectionLimit},
    include_projection_candidates::{CandidateAggregate, aggregate},
    include_resolution_state::ResolutionUsage,
};

const MAX_PROJECTED_IDENTITIES: usize = 64;

pub(super) type FactKey = (String, Option<zrail_core::SourceSpan>, SyntaxGuard);
pub(super) type CallSite = (Option<zrail_core::SourceSpan>, String, SyntaxGuard);

pub(super) struct FactProjection {
    pub(super) additions: Vec<ObservedFact>,
    pub(super) qualities: BTreeMap<FactKey, AnalysisQuality>,
    pub(super) removals: BTreeSet<FactKey>,
}

pub(super) struct ProjectionRequest<'a> {
    pub(super) bindings: &'a IncludeBindings,
    pub(super) file: &'a str,
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
) -> Result<FactProjection, ProjectionLimit> {
    let existing = request
        .facts
        .iter()
        .map(|fact| ((fact.name.clone(), fact.span, fact.guard), fact.quality))
        .collect::<BTreeMap<_, _>>();
    let mut additions = BTreeMap::<FactKey, ObservedFact>::new();
    let mut qualities = BTreeMap::<FactKey, AnalysisQuality>::new();
    let mut removals = BTreeSet::new();
    for fact in request.facts.iter().filter(|fact| fact.written.is_some()) {
        let usage = fact_usage(fact, request.usage, request.call_sites);
        let instances = request.bindings.active_instances(request.file, fact.guard);
        if instances.is_empty() {
            continue;
        }
        budget.consume_work()?;
        let (aggregate, compatible, test_coverage) =
            aggregate(request.bindings, fact, &instances, usage, budget)?;
        if aggregate.len() > MAX_PROJECTED_IDENTITIES {
            *uncertain = uncertain.or(fact.span);
            continue;
        }
        let authoritative = compatible
            && aggregate.len() == 1
            && aggregate.values().all(|candidate| {
                candidate.instances == instances.len()
                    && candidate.quality != AnalysisQuality::Unresolved
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
                    .map(|stale| (stale.name.clone(), stale.span, stale.guard)),
            );
        }
        let mut retention = RetentionState {
            project_expression: request.project_expression,
            existing: &existing,
            additions: &mut additions,
            qualities: &mut qualities,
            uncertain,
            budget,
            remaining_file_facts,
        };
        retain_candidates(
            fact,
            aggregate,
            compatible,
            instances.len(),
            test_coverage,
            &mut retention,
        )?;
    }
    Ok(FactProjection {
        additions: additions.into_values().collect(),
        qualities,
        removals,
    })
}

fn fact_usage(
    fact: &ObservedFact,
    usage: ResolutionUsage,
    call_sites: &BTreeSet<CallSite>,
) -> ResolutionUsage {
    if fact.namespace == super::FactNamespace::Type {
        ResolutionUsage::Type
    } else if usage == ResolutionUsage::Path
        && (fact.namespace == super::FactNamespace::Value
            || call_sites.contains(&(
                fact.span,
                fact.written.as_deref().unwrap_or(&fact.name).to_owned(),
                fact.guard,
            )))
    {
        ResolutionUsage::Call
    } else {
        usage
    }
}

struct RetentionState<'a> {
    project_expression: bool,
    existing: &'a BTreeMap<FactKey, AnalysisQuality>,
    additions: &'a mut BTreeMap<FactKey, ObservedFact>,
    qualities: &'a mut BTreeMap<FactKey, AnalysisQuality>,
    uncertain: &'a mut Option<zrail_core::SourceSpan>,
    budget: &'a mut ProjectionBudget,
    remaining_file_facts: &'a mut usize,
}

fn retain_candidates(
    fact: &ObservedFact,
    aggregate: BTreeMap<String, CandidateAggregate>,
    compatible: bool,
    instance_count: usize,
    test_coverage: super::include_projection_candidates::TestCoverage,
    state: &mut RetentionState<'_>,
) -> Result<(), ProjectionLimit> {
    for (name, candidate) in aggregate {
        let complete = (compatible && candidate.instances == instance_count)
            || (!candidate.production
                && test_coverage.instances > 0
                && test_coverage.compatible
                && candidate.test_instances == test_coverage.instances);
        let quality = if candidate.quality == AnalysisQuality::Unresolved {
            if candidate.requires_projection {
                *state.uncertain = (*state.uncertain).or(fact.span);
            }
            AnalysisQuality::Unresolved
        } else {
            candidate.quality.max(if complete {
                AnalysisQuality::Exact
            } else {
                AnalysisQuality::Conservative
            })
        }
        .max(fact.quality);
        if fact.quality == AnalysisQuality::Unresolved {
            *state.uncertain = (*state.uncertain).or(fact.span);
        }
        let guard = if candidate.production {
            fact.guard
        } else {
            fact.guard.combine(SyntaxGuard::TestOnly)
        };
        if name == fact.name
            && guard == fact.guard
            && quality == fact.quality
            && !candidate.requires_projection
            && complete
            && !state.project_expression
        {
            continue;
        }
        let key = (name.clone(), fact.span, guard);
        if state.existing.contains_key(&key) {
            state
                .qualities
                .entry(key)
                .and_modify(|existing| *existing = (*existing).max(quality))
                .or_insert(quality);
            continue;
        }
        if let Some(existing) = state.additions.get_mut(&key) {
            existing.quality = existing.quality.max(quality);
            continue;
        }
        state.budget.retain_fact(state.remaining_file_facts)?;
        state.additions.insert(
            key,
            ObservedFact {
                name,
                written: None,
                canonical: Vec::new(),
                span: fact.span,
                quality,
                guard,
                lexical_scope: fact.lexical_scope.clone(),
                namespace: fact.namespace,
            },
        );
    }
    Ok(())
}
