//! Candidate aggregation preserves per-domain compatibility and quality.

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::AnalysisQuality;

use super::{
    ObservedFact, SourceInstanceId, SyntaxGuard,
    include_bindings::{IncludeBindings, ResolvedPath},
    include_projection_budget::{ProjectionBudget, ProjectionLimit},
    include_resolution_state::{ResolutionUsage, WrittenResolveRequest},
};

pub(super) type ResolutionCache = BTreeMap<ResolutionCacheKey, Vec<ResolvedPath>>;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct ResolutionCacheKey {
    instance: SourceInstanceId,
    written: String,
    scope: Vec<zrail_core::SourceSpan>,
    usage: ResolutionUsage,
    guard: SyntaxGuard,
}

pub(super) struct CandidateAggregate {
    pub(super) instances: usize,
    pub(super) test_instances: usize,
    pub(super) quality: AnalysisQuality,
    pub(super) production: bool,
    pub(super) requires_projection: bool,
    pub(super) blocks_completeness: bool,
}

impl Default for CandidateAggregate {
    fn default() -> Self {
        Self {
            instances: 0,
            test_instances: 0,
            quality: AnalysisQuality::Exact,
            production: false,
            requires_projection: false,
            blocks_completeness: false,
        }
    }
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
        let key = ResolutionCacheKey {
            instance: *instance,
            written: written.into(),
            scope: fact.lexical_scope.clone(),
            usage,
            guard: fact.guard.clone(),
        };
        let mut resolved = if let Some(resolved) = cache.get(&key) {
            resolved.clone()
        } else {
            let mut seen = BTreeSet::new();
            let resolved = bindings.resolve_written(
                &WrittenResolveRequest {
                    instance: *instance,
                    written,
                    scope: &fact.lexical_scope,
                    depth: 0,
                    usage,
                    guard: &fact.guard,
                    allow_implicit_prelude: false,
                },
                &mut seen,
                budget,
            )?;
            cache.insert(key, resolved.clone());
            resolved
        };
        let source = bindings.instances.get(*instance);
        if source.is_some_and(|source| generic_root(fact, &source.generic_types)) {
            for candidate in &mut resolved {
                candidate.quality = AnalysisQuality::Unresolved;
                candidate.requires_projection = true;
                candidate.blocks_completeness = true;
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
            entry.blocks_completeness |= candidate.blocks_completeness;
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
