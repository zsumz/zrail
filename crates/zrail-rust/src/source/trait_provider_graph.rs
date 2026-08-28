//! Bounded per-domain trait inheritance graph construction.

use std::collections::BTreeMap;

use zrail_core::AnalysisQuality;

use super::super::{
    CompilationDomain, SourceIndex,
    include_bindings::{IncludeBindings, ResolvedPath},
    include_projection_budget::{ProjectionBudget, ProjectionLimit},
    include_resolution_state::{ResolutionTrail, ResolutionUsage, WrittenResolveRequest},
    model::TraitInheritanceFact,
};

pub(super) type ProviderGraph =
    BTreeMap<(CompilationDomain, String), BTreeMap<String, AnalysisQuality>>;

pub(super) fn build(
    index: &SourceIndex,
    bindings: &IncludeBindings,
    budget: &mut ProjectionBudget,
) -> Result<ProviderGraph, ProjectionLimit> {
    let mut graph = ProviderGraph::new();
    for file in &index.files {
        for fact in &file.trait_inheritance {
            for instance in bindings.active_instances(&file.relative, &fact.guard) {
                let Some(source) = bindings.instances.get(*instance) else {
                    continue;
                };
                let traits = resolve(bindings, *instance, &fact.trait_path, fact, budget)?;
                let providers = fact
                    .providers
                    .iter()
                    .map(|provider| resolve(bindings, *instance, provider, fact, budget))
                    .collect::<Result<Vec<_>, _>>()?;
                for identity in traits {
                    add_edges(
                        &mut graph,
                        source.domain.clone(),
                        identity,
                        &providers,
                        fact.quality,
                    );
                }
            }
        }
    }
    close(&mut graph, budget)?;
    Ok(graph)
}

fn add_edges(
    graph: &mut ProviderGraph,
    domain: CompilationDomain,
    identity: ResolvedPath,
    providers: &[Vec<ResolvedPath>],
    fact_quality: AnalysisQuality,
) {
    if identity.quality == AnalysisQuality::Unresolved {
        return;
    }
    let entry = graph.entry((domain, identity.name)).or_default();
    for provider in providers.iter().flatten() {
        if provider.quality == AnalysisQuality::Unresolved {
            continue;
        }
        let edge_quality = fact_quality.max(identity.quality).max(provider.quality);
        entry
            .entry(provider.name.clone())
            .and_modify(|quality| *quality = (*quality).max(edge_quality))
            .or_insert(edge_quality);
    }
}

fn resolve(
    bindings: &IncludeBindings,
    instance: super::super::SourceInstanceId,
    written: &str,
    fact: &TraitInheritanceFact,
    budget: &mut ProjectionBudget,
) -> Result<Vec<ResolvedPath>, ProjectionLimit> {
    bindings.resolve_written(
        &WrittenResolveRequest {
            instance,
            written,
            scope: &fact.lexical_scope,
            depth: 0,
            usage: ResolutionUsage::Type,
            guard: &fact.guard,
            allow_implicit_prelude: true,
        },
        &mut ResolutionTrail::new(),
        budget,
    )
}

fn close(graph: &mut ProviderGraph, budget: &mut ProjectionBudget) -> Result<(), ProjectionLimit> {
    loop {
        let snapshot = graph.clone();
        let mut changed = false;
        for ((domain, _), providers) in graph.iter_mut() {
            let nested = transitive_edges(domain, providers, &snapshot);
            for (provider, quality) in nested {
                budget.consume_work()?;
                changed |= merge_edge(providers, provider, quality);
            }
        }
        if !changed {
            return Ok(());
        }
    }
}

fn transitive_edges(
    domain: &CompilationDomain,
    providers: &BTreeMap<String, AnalysisQuality>,
    graph: &ProviderGraph,
) -> Vec<(String, AnalysisQuality)> {
    providers
        .iter()
        .filter_map(|(provider, quality)| {
            graph
                .get(&(domain.clone(), provider.clone()))
                .map(|nested| (quality, nested))
        })
        .flat_map(|(edge_quality, nested)| {
            nested.iter().map(|(provider, nested_quality)| {
                (provider.clone(), (*edge_quality).max(*nested_quality))
            })
        })
        .collect()
}

fn merge_edge(
    providers: &mut BTreeMap<String, AnalysisQuality>,
    provider: String,
    quality: AnalysisQuality,
) -> bool {
    use std::collections::btree_map::Entry;

    match providers.entry(provider) {
        Entry::Vacant(entry) => {
            entry.insert(quality);
            true
        }
        Entry::Occupied(mut entry) => {
            let combined = (*entry.get()).max(quality);
            if combined == *entry.get() {
                false
            } else {
                entry.insert(combined);
                true
            }
        }
    }
}
