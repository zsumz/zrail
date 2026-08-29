//! Provider queries keep graph storage private and domain-specific.

use crate::source::{CompilationDomain, ProjectionIdentity};

use super::{ProviderEdge, ProviderEdges, ProviderGraph};
impl ProviderGraph {
    pub(in crate::source) fn providers(
        &self,
        domain: &CompilationDomain,
        trait_path: &str,
        projection: &ProjectionIdentity,
    ) -> Option<ProviderEdges> {
        matching_edges(&self.edges, domain, trait_path, projection)
    }

    pub(in crate::source) fn substitutions(
        &self,
        domain: &CompilationDomain,
        trait_path: &str,
        projection: &ProjectionIdentity,
    ) -> Option<ProviderEdges> {
        matching_edges(&self.substitutions, domain, trait_path, projection)
    }

    pub(in crate::source) fn complete(
        &self,
        domain: &CompilationDomain,
        trait_path: &str,
        projection: &ProjectionIdentity,
    ) -> bool {
        let mut found = false;
        for (_, quality) in self.declarations.iter().filter(
            |((candidate_domain, candidate_trait, candidate_projection), _)| {
                candidate_domain == domain
                    && candidate_trait == trait_path
                    && candidate_projection.matches(projection)
            },
        ) {
            found = true;
            if *quality == zrail_core::AnalysisQuality::Unresolved {
                return false;
            }
        }
        found
    }
}

fn matching_edges(
    graph: &std::collections::BTreeMap<super::EdgeKey, ProviderEdges>,
    domain: &CompilationDomain,
    trait_path: &str,
    projection: &ProjectionIdentity,
) -> Option<ProviderEdges> {
    let mut merged = ProviderEdges::new();
    for (_, edges) in graph.iter().filter(
        |((candidate_domain, candidate_trait, candidate_projection), _)| {
            candidate_domain == domain
                && candidate_trait == trait_path
                && candidate_projection.matches(projection)
        },
    ) {
        for (name, edge) in edges {
            merge(&mut merged, name, edge);
        }
    }
    (!merged.is_empty()).then_some(merged)
}

fn merge(edges: &mut ProviderEdges, name: &str, incoming: &ProviderEdge) {
    edges
        .entry(name.into())
        .and_modify(|existing| {
            existing.quality = existing.quality.max(incoming.quality);
            existing
                .authorities
                .extend(incoming.authorities.iter().cloned());
        })
        .or_insert_with(|| incoming.clone());
}
