//! Bounded reachability propagation for exact Rust module-visibility edges.

use std::collections::{BTreeMap, BTreeSet};

use super::Reachability;

const MAX_EDGES_PER_MODULE: usize = 4;

pub(super) type Edges = BTreeMap<String, Reachability>;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct VisibilityNode {
    pub(super) file: String,
    pub(super) reachability: Reachability,
}

pub(super) fn intersect_edges(edges: &Edges, reachability: Reachability) -> Edges {
    let mut compatible = Edges::new();
    for (file, edge_reachability) in edges {
        insert_reachable(
            &mut compatible,
            file,
            reachability.intersection(*edge_reachability),
        );
    }
    compatible
}

pub(super) fn insert_reachable(edges: &mut Edges, file: &str, reachability: Reachability) {
    if reachability.is_unreachable() {
        return;
    }
    edges
        .entry(file.to_owned())
        .and_modify(|current| *current = current.join(reachability))
        .or_insert(reachability);
}

pub(super) fn bounded_nodes(edges: Edges) -> Option<Vec<VisibilityNode>> {
    (edges.len() <= MAX_EDGES_PER_MODULE).then(|| {
        edges
            .into_iter()
            .map(|(file, reachability)| VisibilityNode { file, reachability })
            .collect()
    })
}

pub(super) fn insert_edge_bounded<K: Ord + Clone>(
    map: &mut BTreeMap<K, Edges>,
    overflow: &mut BTreeSet<K>,
    key: K,
    file: &str,
    reachability: Reachability,
) {
    if overflow.contains(&key) {
        return;
    }
    let edges = map.entry(key.clone()).or_default();
    if let Some(current) = edges.get_mut(file) {
        *current = current.join(reachability);
    } else if edges.len() < MAX_EDGES_PER_MODULE {
        edges.insert(file.to_owned(), reachability);
    } else {
        map.remove(&key);
        overflow.insert(key);
    }
}
