//! Trait inheritance expands generic associated identity to plausible providers.

use std::collections::{BTreeMap, BTreeSet};

use syn::Item;
use zrail_core::{AnalysisQuality, Finding};

use super::model::TraitDeclarationFact;
use super::{
    AssociatedCandidateKind, CompilationDomain, GenericAssociatedCandidate, ProjectionIdentity,
    ProviderAuthority, SourceIndex, SyntaxGuard,
    fact::source_span,
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
) -> Vec<TraitDeclarationFact> {
    items
        .filter_map(|item| {
            let Item::Trait(item) = item else {
                return None;
            };
            let guard = item_guard(&item.attrs, enclosing_guard);
            let macros = replacement_macros(&item.attrs, &guard, scope);
            let declaration_quality = if macros.is_empty() {
                quality(&item.attrs)
            } else {
                AnalysisQuality::Unresolved
            };
            let mut bounds =
                super::trait_bounds::from_generics(&item.generics, true, &guard, scope);
            bounds.extend(super::trait_bounds::from_bounds(
                &super::BoundSubject::SelfType,
                &item.supertraits,
                &guard,
                scope,
                source_span(item.ident.span()),
            ));
            bounds.extend(super::trait_bounds::associated_types(item, &guard, scope));
            bounds.retain(|bound| bound.subject.root() == "Self");
            for bound in &mut bounds {
                bound.quality = bound.quality.max(declaration_quality);
            }
            super::trait_bounds::normalize(&mut bounds);
            Some(TraitDeclarationFact {
                trait_path: item.ident.to_string(),
                bounds,
                quality: declaration_quality,
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
            file.trait_declarations.len()
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
    let mut expanded = BTreeMap::<
        (
            String,
            Vec<String>,
            ProjectionIdentity,
            AssociatedCandidateKind,
        ),
        GenericAssociatedCandidate,
    >::new();
    for mut candidate in std::mem::take(candidates) {
        if candidate.kind == AssociatedCandidateKind::TypeEquality {
            insert_candidate(&mut expanded, candidate);
            continue;
        }
        let Some((trait_path, item)) = candidate.name.rsplit_once("::") else {
            insert_candidate(&mut expanded, candidate);
            continue;
        };
        if candidate
            .projection
            .qualifying_trait
            .as_ref()
            .is_some_and(|qualifier| qualifier.path != trait_path)
        {
            continue;
        }
        let base_complete = !domains.is_empty()
            && domains
                .iter()
                .all(|domain| graph.complete(domain, trait_path, &candidate.projection));
        for domain in domains {
            for (provider, provider_edge) in graph
                .providers(domain, trait_path, &candidate.projection)
                .into_iter()
                .flatten()
            {
                let provider_complete =
                    graph.complete(domain, &provider, &ProjectionIdentity::default());
                let mut provider_authorities = provider_edge.authorities.clone();
                if !provider_complete {
                    provider_authorities.insert(ProviderAuthority::Unknown);
                }
                insert_candidate(
                    &mut expanded,
                    GenericAssociatedCandidate {
                        name: format!("{provider}::{item}"),
                        canonical: Vec::new(),
                        quality: candidate.quality.max(provider_edge.quality),
                        projection: ProjectionIdentity::default(),
                        kind: AssociatedCandidateKind::TraitProvider,
                        provider_complete,
                        provider_authorities,
                    },
                );
            }
            for (target, target_edge) in graph
                .substitutions(domain, trait_path, &candidate.projection)
                .into_iter()
                .flatten()
            {
                insert_candidate(
                    &mut expanded,
                    GenericAssociatedCandidate {
                        name: format!("{target}::{item}"),
                        canonical: Vec::new(),
                        quality: candidate.quality.max(target_edge.quality),
                        projection: ProjectionIdentity::default(),
                        kind: AssociatedCandidateKind::TypeEquality,
                        provider_complete: target_edge.quality != AnalysisQuality::Unresolved,
                        provider_authorities: target_edge.authorities,
                    },
                );
            }
        }
        if candidate.projection.is_empty() || !base_complete {
            candidate.provider_complete &= base_complete;
            if !base_complete {
                candidate.quality = AnalysisQuality::Unresolved;
                candidate
                    .provider_authorities
                    .insert(ProviderAuthority::Unknown);
            }
            insert_candidate(&mut expanded, candidate);
        }
    }
    *candidates = expanded.into_values().collect();
}

fn insert_candidate(
    candidates: &mut BTreeMap<
        (
            String,
            Vec<String>,
            ProjectionIdentity,
            AssociatedCandidateKind,
        ),
        GenericAssociatedCandidate,
    >,
    candidate: GenericAssociatedCandidate,
) {
    let key = (
        candidate.name.clone(),
        candidate.canonical.clone(),
        candidate.projection.clone(),
        candidate.kind,
    );
    candidates
        .entry(key)
        .and_modify(|existing| {
            existing.quality = existing.quality.max(candidate.quality);
            existing.provider_complete &= candidate.provider_complete;
            existing
                .provider_authorities
                .extend(candidate.provider_authorities.iter().cloned());
        })
        .or_insert(candidate);
}
