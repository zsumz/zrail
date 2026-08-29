//! Provider-edge closure preserves final authority provenance.

use std::collections::BTreeMap;

use super::{EdgeKey, ProviderEdge, ProviderEdges};
use crate::source::include_bindings::{ResolvedOrigin, ResolvedPath};
use crate::source::{CompilationDomain, ProjectionIdentity, ProviderAuthority};

pub(super) fn inherited_projection(
    domain: &CompilationDomain,
    trait_path: &str,
    projection: &ProjectionIdentity,
    graph: &BTreeMap<EdgeKey, ProviderEdges>,
) -> Vec<(String, ProviderEdge)> {
    if projection.is_empty() {
        return Vec::new();
    }
    graph
        .get(&(
            domain.clone(),
            trait_path.into(),
            ProjectionIdentity::default(),
        ))
        .into_iter()
        .flatten()
        .filter_map(|(provider, edge)| {
            graph
                .get(&(domain.clone(), provider.clone(), projection.clone()))
                .map(|nested| (edge.quality, nested))
        })
        .flat_map(|(quality, nested)| {
            nested.iter().map(move |(provider, nested)| {
                let mut nested = nested.clone();
                nested.quality = quality.max(nested.quality);
                (provider.clone(), nested)
            })
        })
        .collect()
}

pub(super) fn provider_supertraits(
    domain: &CompilationDomain,
    providers: &ProviderEdges,
    graph: &BTreeMap<EdgeKey, ProviderEdges>,
) -> Vec<(String, ProviderEdge)> {
    providers
        .iter()
        .filter_map(|(provider, edge)| {
            graph
                .get(&(
                    domain.clone(),
                    provider.clone(),
                    ProjectionIdentity::default(),
                ))
                .map(|nested| (edge.quality, nested))
        })
        .flat_map(|(quality, nested)| {
            nested.iter().map(move |(provider, nested)| {
                let mut nested = nested.clone();
                nested.quality = quality.max(nested.quality);
                (provider.clone(), nested)
            })
        })
        .collect()
}

pub(super) fn merge_edge(
    edges: &mut ProviderEdges,
    provider: String,
    incoming: ProviderEdge,
) -> bool {
    match edges.entry(provider) {
        std::collections::btree_map::Entry::Vacant(entry) => {
            entry.insert(incoming);
            true
        }
        std::collections::btree_map::Entry::Occupied(mut entry) => {
            let existing = entry.get_mut();
            let original = (existing.quality, existing.authorities.len());
            existing.quality = existing.quality.max(incoming.quality);
            existing.authorities.extend(incoming.authorities);
            original != (existing.quality, existing.authorities.len())
        }
    }
}

pub(super) fn authority(provider: &ResolvedPath) -> ProviderAuthority {
    match provider.origin {
        ResolvedOrigin::CrateLocal => ProviderAuthority::LocalCrate,
        ResolvedOrigin::Unknown => ProviderAuthority::Unknown,
        ResolvedOrigin::External => provider
            .name
            .trim_start_matches("::")
            .split("::")
            .next()
            .filter(|root| !root.is_empty())
            .map_or(ProviderAuthority::Unknown, |root| {
                ProviderAuthority::ExternalRoot(root.strip_prefix("r#").unwrap_or(root).into())
            }),
    }
}
