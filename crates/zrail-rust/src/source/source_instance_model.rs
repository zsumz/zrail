//! Typed source-context incompleteness and deterministic traversal metrics.

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
