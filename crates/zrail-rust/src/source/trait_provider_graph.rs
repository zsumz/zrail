//! Bounded per-domain graph for supertraits and associated-type providers.

#[path = "trait_provider_graph/access.rs"]
mod access;
#[path = "trait_provider_graph/closure.rs"]
mod closure;

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::AnalysisQuality;

use super::super::{
    BoundSubject, CompilationDomain, ProviderAuthority, SourceIndex, TraitBoundFact,
    include_bindings::{IncludeBindings, ResolvedOrigin, ResolvedPath},
    include_projection_budget::{ProjectionBudget, ProjectionLimit},
    include_resolution_state::{ResolutionTrail, ResolutionUsage, WrittenResolveRequest},
    model::TraitDeclarationFact,
};
use closure::{authority, inherited_projection, merge_edge, provider_supertraits};

#[derive(Clone)]
pub(in crate::source) struct ProviderEdge {
    pub(in crate::source) quality: AnalysisQuality,
    pub(in crate::source) authorities: BTreeSet<ProviderAuthority>,
}

type ProviderEdges = BTreeMap<String, ProviderEdge>;
type EdgeKey = (CompilationDomain, String, Vec<String>);

#[derive(Default)]
pub(super) struct ProviderGraph {
    edges: BTreeMap<EdgeKey, ProviderEdges>,
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
                            .entry((source.domain.clone(), identity.name.clone(), Vec::new()))
                            .and_modify(|current| *current = (*current).max(quality))
                            .or_insert(quality);
                    }
                    for bound in &declaration.bounds {
                        let (providers, providers_complete) =
                            resolve_bound(bindings, instance, bound, budget)?;
                        if identity.origin == ResolvedOrigin::CrateLocal {
                            graph.declare_bound(
                                &source.domain,
                                identity,
                                bound,
                                providers_complete,
                            );
                        }
                        graph.add_bound(&source.domain, identity, bound, &providers);
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
        providers_complete: bool,
    ) {
        let projection = match &bound.subject {
            BoundSubject::SelfType => Vec::new(),
            BoundSubject::Projection { associated, .. } => associated.clone(),
            BoundSubject::TypeParameter(_) => return,
        };
        let mut quality = identity.quality.max(bound.quality);
        if !providers_complete {
            quality = AnalysisQuality::Unresolved;
        }
        self.declarations
            .entry((domain.clone(), identity.name.clone(), projection))
            .and_modify(|current| *current = (*current).max(quality))
            .or_insert(quality);
    }

    fn add_bound(
        &mut self,
        domain: &CompilationDomain,
        identity: &ResolvedPath,
        bound: &TraitBoundFact,
        providers: &[ResolvedPath],
    ) {
        let projection = match &bound.subject {
            BoundSubject::SelfType => Vec::new(),
            BoundSubject::Projection { associated, .. } => associated.clone(),
            BoundSubject::TypeParameter(_) => return,
        };
        let entry = self
            .edges
            .entry((domain.clone(), identity.name.clone(), projection))
            .or_default();
        for provider in providers
            .iter()
            .filter(|provider| provider.quality != AnalysisQuality::Unresolved)
        {
            let quality = bound.quality.max(identity.quality).max(provider.quality);
            merge_edge(
                entry,
                provider.name.clone(),
                ProviderEdge {
                    quality,
                    authorities: [authority(provider)].into(),
                },
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

fn resolve_declaration(
    bindings: &IncludeBindings,
    instance: super::super::SourceInstanceId,
    declaration: &TraitDeclarationFact,
    budget: &mut ProjectionBudget,
) -> Result<Vec<ResolvedPath>, ProjectionLimit> {
    resolve(
        bindings,
        instance,
        &declaration.trait_path,
        &declaration.lexical_scope,
        &declaration.guard,
        budget,
    )
}

fn resolve_bound(
    bindings: &IncludeBindings,
    instance: super::super::SourceInstanceId,
    bound: &TraitBoundFact,
    budget: &mut ProjectionBudget,
) -> Result<(Vec<ResolvedPath>, bool), ProjectionLimit> {
    let mut resolved = Vec::new();
    let mut complete = true;
    for provider in &bound.providers {
        let candidates = resolve(
            bindings,
            instance,
            provider,
            &bound.lexical_scope,
            &bound.guard,
            budget,
        )?;
        complete &= !candidates.is_empty()
            && candidates
                .iter()
                .all(|candidate| candidate.quality != AnalysisQuality::Unresolved);
        resolved.extend(candidates);
    }
    Ok((resolved, complete))
}

fn resolve(
    bindings: &IncludeBindings,
    instance: super::super::SourceInstanceId,
    written: &str,
    scope: &[zrail_core::SourceSpan],
    guard: &super::super::SyntaxGuard,
    budget: &mut ProjectionBudget,
) -> Result<Vec<ResolvedPath>, ProjectionLimit> {
    bindings.resolve_written(
        &WrittenResolveRequest {
            instance,
            written,
            scope,
            depth: 0,
            usage: ResolutionUsage::Type,
            guard,
            allow_implicit_prelude: true,
        },
        &mut ResolutionTrail::new(),
        budget,
    )
}
