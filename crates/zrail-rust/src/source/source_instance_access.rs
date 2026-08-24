//! Bounded issue retention keeps adversarial traversal diagnostics finite.

use super::{
    CompilationIncludeEdge, CompilationModuleEdge, SourceEntry, SourceInstance, SourceInstanceId,
    SourceInstanceIssue, SourceInstances,
};

impl SourceInstances {
    pub(crate) fn get(&self, id: SourceInstanceId) -> Option<&SourceInstance> {
        self.instances.get(id.0)
    }

    pub(crate) fn for_file(&self, file: &str) -> &[SourceInstanceId] {
        self.by_file.get(file).map_or(&[], Vec::as_slice)
    }

    pub(crate) fn includes_from(
        &self,
        parent: SourceInstanceId,
    ) -> &[(CompilationIncludeEdge, SourceInstanceId)] {
        self.include_children
            .get(&parent)
            .map_or(&[], Vec::as_slice)
    }

    pub(crate) fn modules_from(
        &self,
        parent: SourceInstanceId,
    ) -> &[(CompilationModuleEdge, SourceInstanceId)] {
        self.module_children.get(&parent).map_or(&[], Vec::as_slice)
    }

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

    pub(super) fn ancestor_contains(&self, mut parent: SourceInstanceId, file: &str) -> bool {
        loop {
            let instance = &self.instances[parent.0];
            if instance.file == file {
                return true;
            }
            let Some(next) = instance.parent else {
                return false;
            };
            parent = next;
        }
    }

    pub(super) fn ancestor_chain(&self, mut current: SourceInstanceId) -> Vec<String> {
        let mut chain = Vec::new();
        loop {
            let instance = &self.instances[current.0];
            chain.push(instance.file.clone());
            let Some(parent) = instance.parent else {
                break;
            };
            current = parent;
        }
        chain.reverse();
        chain
    }

    fn has_include_ancestor(&self, mut current: SourceInstanceId) -> bool {
        loop {
            let instance = &self.instances[current.0];
            if matches!(instance.entered_from, SourceEntry::Include(_)) {
                return true;
            }
            let Some(parent) = instance.parent else {
                return false;
            };
            current = parent;
        }
    }
}
