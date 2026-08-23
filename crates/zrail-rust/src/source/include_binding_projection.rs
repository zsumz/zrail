//! Transactional projection replaces stale physical candidates with typed identities.

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::AnalysisQuality;

use super::{
    ObservedFact, SyntaxGuard,
    include_bindings::IncludeBindings,
    include_projection_budget::{ProjectionBudget, ProjectionLimit},
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

struct CandidateAggregate {
    instances: usize,
    quality: AnalysisQuality,
    production: bool,
    requires_projection: bool,
}

impl Default for CandidateAggregate {
    fn default() -> Self {
        Self {
            instances: 0,
            quality: AnalysisQuality::Exact,
            production: false,
            requires_projection: false,
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn project(
    bindings: &IncludeBindings,
    file: &str,
    facts: &[ObservedFact],
    usage: ResolutionUsage,
    call_sites: &BTreeSet<CallSite>,
    project_expression: bool,
    uncertain: &mut Option<zrail_core::SourceSpan>,
    budget: &mut ProjectionBudget,
    remaining_file_facts: &mut usize,
) -> Result<FactProjection, ProjectionLimit> {
    let existing = facts
        .iter()
        .map(|fact| ((fact.name.clone(), fact.span, fact.guard), fact.quality))
        .collect::<BTreeMap<_, _>>();
    let mut additions = BTreeMap::<FactKey, ObservedFact>::new();
    let mut qualities = BTreeMap::<FactKey, AnalysisQuality>::new();
    let mut removals = BTreeSet::new();
    for fact in facts.iter().filter(|fact| fact.written.is_some()) {
        budget.consume_work()?;
        let usage = fact_usage(fact, usage, call_sites);
        let instances = bindings.active_instances(file, fact.guard, budget)?;
        if instances.is_empty() {
            continue;
        }
        let (aggregate, compatible) = aggregate(bindings, fact, &instances, usage, budget)?;
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
                facts
                    .iter()
                    .filter(|stale| {
                        stale.span == fact.span
                            && stale.guard == fact.guard
                            && !aggregate.contains_key(&stale.name)
                    })
                    .map(|stale| (stale.name.clone(), stale.span, stale.guard)),
            );
        }
        retain_candidates(
            fact,
            aggregate,
            compatible,
            instances.len(),
            project_expression,
            &existing,
            &mut additions,
            &mut qualities,
            uncertain,
            budget,
            remaining_file_facts,
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

fn aggregate(
    bindings: &IncludeBindings,
    fact: &ObservedFact,
    instances: &[super::SourceInstanceId],
    usage: ResolutionUsage,
    budget: &mut ProjectionBudget,
) -> Result<(BTreeMap<String, CandidateAggregate>, bool), ProjectionLimit> {
    let mut aggregate = BTreeMap::<String, CandidateAggregate>::new();
    let mut compatible = true;
    let mut common = None;
    for instance in instances {
        let mut seen = BTreeSet::new();
        let resolved = bindings.resolve_written(
            *instance,
            fact.written.as_deref().unwrap_or(&fact.name),
            &fact.lexical_scope,
            &mut seen,
            0,
            budget,
            usage,
        )?;
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
            let entry = aggregate.entry(candidate.name).or_default();
            entry.instances += 1;
            entry.quality = entry.quality.max(candidate.quality);
            entry.requires_projection |= candidate.requires_projection;
            entry.production |= bindings
                .instances
                .get(*instance)
                .is_some_and(|source| !source.domain.mode.enables_cfg_test());
        }
    }
    Ok((aggregate, compatible))
}

#[allow(clippy::too_many_arguments)]
fn retain_candidates(
    fact: &ObservedFact,
    aggregate: BTreeMap<String, CandidateAggregate>,
    compatible: bool,
    instance_count: usize,
    project_expression: bool,
    existing: &BTreeMap<FactKey, AnalysisQuality>,
    additions: &mut BTreeMap<FactKey, ObservedFact>,
    qualities: &mut BTreeMap<FactKey, AnalysisQuality>,
    uncertain: &mut Option<zrail_core::SourceSpan>,
    budget: &mut ProjectionBudget,
    remaining_file_facts: &mut usize,
) -> Result<(), ProjectionLimit> {
    for (name, candidate) in aggregate {
        let complete = compatible && candidate.instances == instance_count;
        let quality = if candidate.quality == AnalysisQuality::Unresolved {
            if candidate.requires_projection {
                *uncertain = uncertain.or(fact.span);
            }
            AnalysisQuality::Unresolved
        } else if complete {
            AnalysisQuality::Exact
        } else {
            AnalysisQuality::Conservative
        };
        let guard = if fact.guard == SyntaxGuard::TestOnly || !candidate.production {
            SyntaxGuard::TestOnly
        } else {
            SyntaxGuard::Ordinary
        };
        if name == fact.name
            && guard == fact.guard
            && quality == fact.quality
            && !candidate.requires_projection
            && complete
            && !project_expression
        {
            continue;
        }
        let key = (name.clone(), fact.span, guard);
        if existing.contains_key(&key) {
            qualities
                .entry(key)
                .and_modify(|existing| *existing = (*existing).max(quality))
                .or_insert(quality);
            continue;
        }
        if let Some(existing) = additions.get_mut(&key) {
            existing.quality = existing.quality.max(quality);
            continue;
        }
        budget.retain_fact(remaining_file_facts)?;
        additions.insert(
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
