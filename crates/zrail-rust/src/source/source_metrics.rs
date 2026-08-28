//! Source analysis counters are separate from the extracted fact model.

use std::collections::BTreeSet;

use zrail_core::Finding;

use super::RustFileFacts;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct SourceAnalysisMetrics {
    pub(crate) base_contexts: usize,
    pub(crate) derived_contexts: usize,
    pub(crate) projection_files: usize,
    pub(crate) projection_work: usize,
    pub(crate) projected_facts: usize,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct SourceIndex {
    pub(crate) files: Vec<RustFileFacts>,
    pub(crate) findings: Vec<Finding>,
    pub(crate) analysis_metrics: SourceAnalysisMetrics,
}

impl SourceIndex {
    pub(crate) fn physical_paths(&self) -> BTreeSet<&str> {
        self.files
            .iter()
            .map(|file| file.relative.as_str())
            .collect()
    }

    pub(crate) fn physical_file_count(&self) -> usize {
        self.physical_paths().len()
    }
}
