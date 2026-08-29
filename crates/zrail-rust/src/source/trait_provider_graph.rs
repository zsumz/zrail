//! Bounded per-domain graph for supertraits and associated-type providers.

#[path = "trait_provider_graph/access.rs"]
mod access;
#[path = "trait_provider_graph/closure.rs"]
mod closure;
#[path = "trait_provider_graph/resolution.rs"]
mod resolution;

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::AnalysisQuality;

use super::super::{
    BoundSubject, CompilationDomain, ProjectionIdentity, ProviderAuthority, SourceIndex,
    TraitBoundFact,
    include_bindings::{IncludeBindings, ResolvedOrigin, ResolvedPath},
    include_projection_budget::{ProjectionBudget, ProjectionLimit},
};
use closure::{authority, inherited_projection, merge_edge, provider_supertraits};
use resolution::{resolve_bound, resolve_declaration};

#[derive(Clone)]
pub(in crate::source) struct ProviderEdge {
    pub(in crate::source) quality: AnalysisQuality,
    pub(in crate::source) authorities: BTreeSet<ProviderAuthority>,
}

type ProviderEdges = BTreeMap<String, ProviderEdge>;
type EdgeKey = (CompilationDomain, String, ProjectionIdentity);

#[derive(Default)]
pub(super) struct ProviderGraph {
    edges: BTreeMap<EdgeKey, ProviderEdges>,
    substitutions: BTreeMap<EdgeKey, ProviderEdges>,
    declarations: BTreeMap<EdgeKey, AnalysisQuality>,
}

pub(super) fn build(
    index: &SourceIndex,
    bindings: &IncludeBindings,
    budget: &mut ProjectionBudget,
) -> Result<ProviderGraph, ProjectionLimit> {
    let mut graph = ProviderGraph::default();
    for file in &index.files {
        for declaration in &file.trait_declarations {
            for instance in
                bindings.active_instances(&file.relative, file.syntax, &declaration.guard)
            {
                let Some(source) = bindings.instances.get(instance) else {
                    continue;
                };
                let traits = resolve_declaration(bindings, instance, declaration, budget)?;
                for identity in traits
                    .iter()
                    .filter(|identity| identity.quality != AnalysisQuality::Unresolved)
                {
                    let quality = declaration.quality.max(identity.quality);
                    if identity.origin == ResolvedOrigin::CrateLocal {
                        graph
                            .declarations
                            .entry((
                                source.domain.clone(),
                                identity.name.clone(),
                                ProjectionIdentity::default(),
                            ))
                            .and_modify(|current| *current = (*current).max(quality))
                            .or_insert(quality);
                    }
                    for bound in &declaration.bounds {
                        let resolved = resolve_bound(bindings, instance, bound, budget)?;
                        if identity.origin == ResolvedOrigin::CrateLocal {
                            graph.declare_bound(&source.domain, identity, bound, &resolved);
                        }
                        graph.add_bound(&source.domain, identity, bound, &resolved);
                    }
                }
            }
        }
    }
    graph.close(budget)?;
    Ok(graph)
}

impl ProviderGraph {
    fn declare_bound(
        &mut self,
        domain: &CompilationDomain,
        identity: &ResolvedPath,
        bound: &TraitBoundFact,
        resolved: &resolution::ResolvedBound,
    ) {
        let projections = match &bound.subject {
            BoundSubject::SelfType => vec![ProjectionIdentity::default()],
            BoundSubject::Projection { .. } => resolved.projections.clone(),
            BoundSubject::TypeParameter(_) => return,
        };
        let mut quality = identity.quality.max(bound.quality);
        if !resolved.complete {
            quality = AnalysisQuality::Unresolved;
        }
        for projection in projections {
            self.declarations
                .entry((domain.clone(), identity.name.clone(), projection))
                .and_modify(|current| *current = (*current).max(quality))
                .or_insert(quality);
        }
    }

    fn add_bound(
        &mut self,
        domain: &CompilationDomain,
        identity: &ResolvedPath,
        bound: &TraitBoundFact,
        resolved: &resolution::ResolvedBound,
    ) {
        let projections = match &bound.subject {
            BoundSubject::SelfType => vec![ProjectionIdentity::default()],
            BoundSubject::Projection { .. } => resolved.projections.clone(),
            BoundSubject::TypeParameter(_) => return,
        };
        for projection in projections {
            add_edges(
                self.edges
                    .entry((domain.clone(), identity.name.clone(), projection.clone()))
                    .or_default(),
                identity,
                bound,
                &resolved.providers,
            );
            add_edges(
                self.substitutions
                    .entry((domain.clone(), identity.name.clone(), projection))
                    .or_default(),
                identity,
                bound,
                &resolved.equalities,
            );
        }
    }

    fn close(&mut self, budget: &mut ProjectionBudget) -> Result<(), ProjectionLimit> {
        loop {
            let snapshot = self.edges.clone();
            let mut changed = false;
            for ((domain, trait_path, projection), providers) in &mut self.edges {
                let mut additions = inherited_projection(domain, trait_path, projection, &snapshot);
                additions.extend(provider_supertraits(domain, providers, &snapshot));
                for (provider, edge) in additions {
                    budget.consume_work()?;
                    changed |= merge_edge(providers, provider, edge);
                }
            }
            if !changed {
                return Ok(());
            }
        }
    }
}

fn add_edges(
    edges: &mut ProviderEdges,
    identity: &ResolvedPath,
    bound: &TraitBoundFact,
    targets: &[ResolvedPath],
) {
    for target in targets
        .iter()
        .filter(|target| target.quality != AnalysisQuality::Unresolved)
    {
        let quality = bound.quality.max(identity.quality).max(target.quality);
        merge_edge(
            edges,
            target.name.clone(),
            ProviderEdge {
                quality,
                authorities: [authority(target)].into(),
            },
        );
    }
}
