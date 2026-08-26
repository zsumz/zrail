//! Candidate retention keeps projection completeness and fact budgets together.

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::AnalysisQuality;

use super::super::{
    FactNamespace, ObservedFact, SyntaxGuard,
    include_projection_budget::{ProjectionBudget, ProjectionLimit},
    include_projection_candidates::{CandidateAggregate, TestCoverage},
    include_resolution_state::ResolutionUsage,
};
use super::{CallSite, FactKey};

pub(super) fn fact_usage(
    fact: &ObservedFact,
    usage: ResolutionUsage,
    call_sites: &BTreeSet<CallSite>,
) -> ResolutionUsage {
    if fact.namespace == FactNamespace::Type {
        ResolutionUsage::Type
    } else if usage == ResolutionUsage::Path
        && (fact.namespace == FactNamespace::Value
            || call_sites.contains(&(
                fact.span,
                fact.written.as_deref().unwrap_or(&fact.name).to_owned(),
                fact.guard.clone(),
            )))
    {
        ResolutionUsage::Call
    } else {
        usage
    }
}

pub(super) struct RetentionState<'a> {
    pub(super) project_expression: bool,
    pub(super) existing: &'a BTreeMap<FactKey, AnalysisQuality>,
    pub(super) additions: &'a mut BTreeMap<FactKey, ObservedFact>,
    pub(super) qualities: &'a mut BTreeMap<FactKey, AnalysisQuality>,
    pub(super) uncertain: &'a mut Option<zrail_core::SourceSpan>,
    pub(super) budget: &'a mut ProjectionBudget,
    pub(super) remaining_file_facts: &'a mut usize,
}

pub(super) fn retain_candidates(
    fact: &ObservedFact,
    aggregate: BTreeMap<String, CandidateAggregate>,
    compatible: bool,
    instance_count: usize,
    test_coverage: TestCoverage,
    state: &mut RetentionState<'_>,
) -> Result<(), ProjectionLimit> {
    demote_non_authoritative_written_fact(fact, &aggregate, compatible, instance_count, state);
    for (name, candidate) in aggregate {
        let complete = (compatible && candidate.instances == instance_count)
            || (!candidate.production
                && test_coverage.instances > 0
                && test_coverage.compatible
                && candidate.test_instances == test_coverage.instances);
        let resolved_quality = if candidate.quality == AnalysisQuality::Unresolved {
            if candidate.blocks_completeness {
                *state.uncertain = (*state.uncertain).or(fact.span);
            }
            AnalysisQuality::Unresolved
        } else {
            candidate.quality.max(if complete {
                AnalysisQuality::Exact
            } else {
                AnalysisQuality::Conservative
            })
        };
        let exact_repair = compatible && complete && candidate.quality == AnalysisQuality::Exact;
        let quality = if exact_repair {
            resolved_quality
        } else {
            resolved_quality.max(fact.quality)
        };
        let guard = if candidate.production {
            fact.guard.clone()
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
        let key = (name.clone(), fact.span, guard.clone());
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

fn demote_non_authoritative_written_fact(
    fact: &ObservedFact,
    aggregate: &BTreeMap<String, CandidateAggregate>,
    compatible: bool,
    instance_count: usize,
    state: &mut RetentionState<'_>,
) {
    let authoritative = compatible
        && aggregate.len() == 1
        && aggregate.values().all(|candidate| {
            candidate.instances == instance_count
                && candidate.quality != AnalysisQuality::Unresolved
        });
    if authoritative {
        return;
    }
    let quality = aggregate
        .values()
        .map(|candidate| candidate.quality)
        .max()
        .unwrap_or(AnalysisQuality::Unresolved)
        .max(fact.quality);
    let key = (fact.name.clone(), fact.span, fact.guard.clone());
    state
        .qualities
        .entry(key)
        .and_modify(|existing| *existing = (*existing).max(quality))
        .or_insert(quality);
}
