//! Bounded issue retention keeps adversarial traversal diagnostics finite.

use super::{SourceInstanceIssue, SourceInstances};

impl SourceInstances {
    pub(crate) fn issues(&self) -> &[SourceInstanceIssue] {
        &self.issues
    }

    pub(crate) fn is_complete(&self) -> bool {
        self.issues.is_empty()
    }

    pub(crate) const fn metrics(&self) -> super::SourceInstanceMetrics {
        self.metrics
    }

    pub(crate) fn requires_projection(&self, file: &str) -> bool {
        self.for_file(file)
            .iter()
            .copied()
            .any(|id| !self.includes_from(id).is_empty() || self.has_include_ancestor(id))
    }

    pub(super) fn record_issue(&mut self, issue: SourceInstanceIssue) {
        if self.issues.len() < 32 && !self.issues.contains(&issue) {
            self.issues.push(issue);
        }
    }
}
