//! Compilation edges are grouped once before source-context traversal.

use std::collections::BTreeMap;

use super::{CompilationDomain, CompilationIncludeEdge, CompilationModuleEdge};

pub(super) fn grouped_modules(
    edges: &[CompilationModuleEdge],
) -> BTreeMap<(String, CompilationDomain), Vec<&CompilationModuleEdge>> {
    let mut grouped = BTreeMap::new();
    for edge in edges {
        grouped
            .entry((edge.parent.clone(), edge.domain.clone()))
            .or_insert_with(Vec::new)
            .push(edge);
    }
    grouped
}

pub(super) fn grouped_includes(
    edges: &[CompilationIncludeEdge],
) -> BTreeMap<(String, CompilationDomain), Vec<&CompilationIncludeEdge>> {
    let mut grouped = BTreeMap::new();
    for edge in edges {
        grouped
            .entry((edge.parent.clone(), edge.domain.clone()))
            .or_insert_with(Vec::new)
            .push(edge);
    }
    grouped
}
