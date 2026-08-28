//! Trait inheritance expands generic associated identity to plausible providers.

use std::collections::{BTreeMap, BTreeSet};

use syn::{Item, TypeParamBound};
use zrail_core::{AnalysisQuality, Finding};

use super::model::TraitInheritanceFact;
use super::{
    CompilationDomain, GenericAssociatedCandidate, SourceIndex, SyntaxGuard,
    fact::{source_span, written_path},
    include_bindings::IncludeBindings,
    include_projection_budget::{ProjectionBudget, ProjectionLimits},
    ordinary_binding_facts::{item_guard, quality, replacement_macros},
};

#[path = "trait_provider_graph.rs"]
mod provider_graph;
use provider_graph::ProviderGraph;

pub(super) fn collect<'a>(
    items: impl Iterator<Item = &'a Item>,
    enclosing_guard: &SyntaxGuard,
    scope: &[zrail_core::SourceSpan],
) -> Vec<TraitInheritanceFact> {
    items
        .filter_map(|item| {
            let Item::Trait(item) = item else {
                return None;
            };
            let providers = item
                .supertraits
                .iter()
                .filter_map(|bound| match bound {
                    TypeParamBound::Trait(bound)
                        if matches!(bound.modifier, syn::TraitBoundModifier::None) =>
                    {
                        Some(written_path(&bound.path))
                    }
                    _ => None,
                })
                .collect::<Vec<_>>();
            if providers.is_empty() {
                return None;
            }
            let guard = item_guard(&item.attrs, enclosing_guard);
            let macros = replacement_macros(&item.attrs, &guard, scope);
            Some(TraitInheritanceFact {
                trait_path: item.ident.to_string(),
                providers,
                quality: if macros.is_empty() {
                    quality(&item.attrs)
                } else {
                    AnalysisQuality::Unresolved
                },
                guard,
                lexical_scope: scope.to_vec(),
                span: source_span(item.ident.span()),
            })
        })
        .collect()
}

pub(super) fn apply(
    index: &mut SourceIndex,
    bindings: &IncludeBindings,
    domains: &BTreeMap<String, BTreeSet<CompilationDomain>>,
    limits: &zrail_core::AnalysisLimits,
) -> Vec<Finding> {
    let affected = index
        .files
        .iter()
        .map(|file| {
            file.trait_inheritance.len()
                + file
                    .paths
                    .iter()
                    .chain(&file.calls)
                    .map(|fact| fact.associated_candidates.len())
                    .sum::<usize>()
        })
        .sum();
    let metrics = bindings.instances.metrics();
    let mut budget = ProjectionBudget::new(ProjectionLimits::for_contract(
        affected,
        metrics
            .base_contexts
            .saturating_add(metrics.derived_contexts),
        limits,
    ));
    let graph = match provider_graph::build(index, bindings, &mut budget) {
        Ok(graph) => graph,
        Err(limit) => return vec![super::include_projection_apply::budget_exhausted(limit)],
    };
    for file in &mut index.files {
        let Some(file_domains) = domains.get(&file.relative) else {
            continue;
        };
        for fact in file.paths.iter_mut().chain(&mut file.calls) {
            expand_candidates(&mut fact.associated_candidates, file_domains, &graph);
        }
    }
    Vec::new()
}

fn expand_candidates(
    candidates: &mut Vec<GenericAssociatedCandidate>,
    domains: &BTreeSet<CompilationDomain>,
    graph: &ProviderGraph,
) {
    let mut additions = BTreeMap::<String, AnalysisQuality>::new();
    for candidate in candidates.iter() {
        let Some((trait_path, item)) = candidate.name.rsplit_once("::") else {
            continue;
        };
        for domain in domains {
            for (provider, provider_quality) in graph
                .get(&(domain.clone(), trait_path.into()))
                .into_iter()
                .flatten()
            {
                additions
                    .entry(format!("{provider}::{item}"))
                    .and_modify(|quality| {
                        *quality = (*quality).max(candidate.quality).max(*provider_quality);
                    })
                    .or_insert(candidate.quality.max(*provider_quality));
            }
        }
    }
    candidates.extend(
        additions
            .into_iter()
            .map(|(name, quality)| GenericAssociatedCandidate {
                name,
                canonical: Vec::new(),
                quality,
            }),
    );
    candidates.sort();
    candidates.dedup();
}
