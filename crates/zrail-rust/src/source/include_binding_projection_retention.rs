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
    pub(super) associated_candidates:
        &'a mut BTreeMap<FactKey, Vec<super::super::GenericAssociatedCandidate>>,
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
        let associated_candidates = candidate
            .associated_candidates
            .values()
            .cloned()
            .collect::<Vec<_>>();
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
            && associated_candidates == fact.associated_candidates
        {
            continue;
        }
        let key = (name.clone(), fact.span, guard.clone());
        if state.existing.contains_key(&key) {
            state
                .qualities
                .entry(key.clone())
                .and_modify(|existing| *existing = (*existing).max(quality))
                .or_insert(quality);
            merge_candidates(state.associated_candidates, key, &associated_candidates);
            continue;
        }
        if let Some(existing) = state.additions.get_mut(&key) {
            existing.quality = existing.quality.max(quality);
            merge_candidate_vec(&mut existing.associated_candidates, &associated_candidates);
            continue;
        }
        state.budget.retain_fact(state.remaining_file_facts)?;
        let retains_generic_occurrence =
            candidate.generic_shadow.is_some() || !associated_candidates.is_empty();
        state.additions.insert(
            key,
            ObservedFact {
                name,
                written: retains_generic_occurrence
                    .then(|| fact.written.clone())
                    .flatten(),
                implicit_prelude: super::super::ImplicitPreludeEligibility::Disabled,
                canonical: Vec::new(),
                span: fact.span,
                quality,
                guard,
                lexical_scope: fact.lexical_scope.clone(),
                namespace: fact.namespace,
                generic_shadow: candidate.generic_shadow,
                associated_candidates,
                inherits_parent_context: fact.inherits_parent_context,
            },
        );
    }
    Ok(())
}

fn merge_candidates(
    updates: &mut BTreeMap<FactKey, Vec<super::super::GenericAssociatedCandidate>>,
    key: FactKey,
    candidates: &[super::super::GenericAssociatedCandidate],
) {
    merge_candidate_vec(updates.entry(key).or_default(), candidates);
}

fn merge_candidate_vec(
    target: &mut Vec<super::super::GenericAssociatedCandidate>,
    candidates: &[super::super::GenericAssociatedCandidate],
) {
    for candidate in candidates {
        if let Some(existing) = target.iter_mut().find(|existing| {
            existing.name == candidate.name
                && existing.projection == candidate.projection
                && existing.kind == candidate.kind
        }) {
            existing.quality = existing.quality.max(candidate.quality);
            existing.provider_complete &= candidate.provider_complete;
            existing
                .provider_authorities
                .extend(candidate.provider_authorities.iter().cloned());
        } else {
            target.push(candidate.clone());
        }
    }
    target.sort();
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
