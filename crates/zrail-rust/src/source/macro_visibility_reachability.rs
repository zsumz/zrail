//! Bounded reachability propagation for exact Rust module-visibility edges.

use std::collections::{BTreeMap, BTreeSet};

use super::{Reachability, SourceSyntax};

const MAX_EDGES_PER_MODULE: usize = 4;

pub(super) type VisibilityKey = (String, SourceSyntax);
pub(super) type Edges = BTreeMap<VisibilityKey, Reachability>;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct VisibilityNode {
    pub(super) file: String,
    pub(super) syntax: SourceSyntax,
    pub(super) reachability: Reachability,
}

pub(super) fn intersect_edges(edges: &Edges, reachability: Reachability) -> Edges {
    let mut compatible = Edges::new();
    for (node, edge_reachability) in edges {
        insert_reachable(
            &mut compatible,
            node,
            reachability.intersection(*edge_reachability),
        );
    }
    compatible
}

pub(super) fn insert_reachable(
    edges: &mut Edges,
    node: &VisibilityKey,
    reachability: Reachability,
) {
    if reachability.is_unreachable() {
        return;
    }
    edges
        .entry(node.clone())
        .and_modify(|current| *current = current.join(reachability))
        .or_insert(reachability);
}

pub(super) fn bounded_nodes(edges: Edges) -> Option<Vec<VisibilityNode>> {
    (edges.len() <= MAX_EDGES_PER_MODULE).then(|| {
        edges
            .into_iter()
            .map(|((file, syntax), reachability)| VisibilityNode {
                file,
                syntax,
                reachability,
            })
            .collect()
    })
}

pub(super) fn insert_edge_bounded<K: Ord + Clone>(
    map: &mut BTreeMap<K, Edges>,
    overflow: &mut BTreeSet<K>,
    key: K,
    node: &VisibilityKey,
    reachability: Reachability,
) {
    if overflow.contains(&key) {
        return;
    }
    let edges = map.entry(key.clone()).or_default();
    if let Some(current) = edges.get_mut(node) {
        *current = current.join(reachability);
    } else if edges.len() < MAX_EDGES_PER_MODULE {
        edges.insert(node.clone(), reachability);
    } else {
        map.remove(&key);
        overflow.insert(key);
    }
}
