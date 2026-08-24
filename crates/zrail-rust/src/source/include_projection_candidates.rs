//! Candidate aggregation preserves per-domain compatibility and quality.

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::AnalysisQuality;

use super::{
    ObservedFact, SourceInstanceId,
    include_bindings::IncludeBindings,
    include_projection_budget::{ProjectionBudget, ProjectionLimit},
    include_resolution_state::ResolutionUsage,
};

pub(super) struct CandidateAggregate {
    pub(super) instances: usize,
    pub(super) test_instances: usize,
    pub(super) quality: AnalysisQuality,
    pub(super) production: bool,
    pub(super) requires_projection: bool,
}

impl Default for CandidateAggregate {
    fn default() -> Self {
        Self {
            instances: 0,
            test_instances: 0,
            quality: AnalysisQuality::Exact,
            production: false,
            requires_projection: false,
        }
    }
}

pub(super) fn aggregate(
    bindings: &IncludeBindings,
    fact: &ObservedFact,
    instances: &[SourceInstanceId],
    usage: ResolutionUsage,
    budget: &mut ProjectionBudget,
) -> Result<(BTreeMap<String, CandidateAggregate>, bool, TestCoverage), ProjectionLimit> {
    let mut aggregate = BTreeMap::<String, CandidateAggregate>::new();
    let mut compatible = true;
    let mut test_coverage = TestCoverage::default();
    let mut common = None;
    for instance in instances {
        let mut seen = BTreeSet::new();
        let mut resolved = bindings.resolve_written(
            *instance,
            fact.written.as_deref().unwrap_or(&fact.name),
            &fact.lexical_scope,
            &mut seen,
            0,
            budget,
            usage,
        )?;
        let source = bindings.instances.get(*instance);
        if source.is_some_and(|source| generic_root(fact, &source.generic_types)) {
            for candidate in &mut resolved {
                candidate.quality = AnalysisQuality::Unresolved;
                candidate.requires_projection = true;
            }
        }
        let test_instance = source.is_some_and(|source| source.domain.mode.enables_cfg_test());
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
            let entry = aggregate.entry(candidate.name).or_default();
            entry.instances += 1;
            entry.test_instances += usize::from(test_instance);
            entry.quality = entry.quality.max(candidate.quality);
            entry.requires_projection |= candidate.requires_projection;
            entry.production |= bindings
                .instances
                .get(*instance)
                .is_some_and(|source| !source.domain.mode.enables_cfg_test());
        }
    }
    Ok((aggregate, compatible, test_coverage))
}

fn generic_root(fact: &ObservedFact, generic_types: &[String]) -> bool {
    let Some(written) = fact.written.as_deref() else {
        return false;
    };
    let root = written.trim_start_matches("::").split("::").next();
    root.is_some_and(|root| generic_types.iter().any(|generic| generic == root))
}

#[derive(Clone, Copy)]
pub(super) struct TestCoverage {
    pub(super) instances: usize,
    pub(super) compatible: bool,
}

impl Default for TestCoverage {
    fn default() -> Self {
        Self {
            instances: 0,
            compatible: true,
        }
    }
}
