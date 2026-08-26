//! Source analysis counters are separate from the extracted fact model.

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct SourceAnalysisMetrics {
    pub(crate) base_contexts: usize,
    pub(crate) derived_contexts: usize,
    pub(crate) projection_files: usize,
    pub(crate) projection_work: usize,
    pub(crate) projected_facts: usize,
}
