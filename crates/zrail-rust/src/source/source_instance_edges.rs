//! Compilation edges are grouped once before source-context traversal.

use std::collections::BTreeMap;

use super::{CompilationDomain, CompilationIncludeEdge, CompilationModuleEdge};

pub(super) const MIN_DERIVED_SOURCE_CONTEXTS: usize = 4096;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum SourceInstanceIssue {
    DerivedContextLimit {
        used: usize,
        limit: usize,
        file: String,
    },
    DepthLimit {
        file: String,
        depth: usize,
        chain: Vec<String>,
    },
    Cycle {
        chain: Vec<String>,
    },
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct SourceInstanceMetrics {
    pub(crate) base_contexts: usize,
    pub(crate) derived_contexts: usize,
}

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
