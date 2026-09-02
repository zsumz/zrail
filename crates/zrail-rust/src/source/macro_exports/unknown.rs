//! Unknown export sets remain scoped by name, visibility, cfg, and reason.

use super::super::{GuardAvailability, SyntaxGuard, logical_modules::LogicalModule};
use super::{ExportVisibility, GlobExport, NamedExport};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct UnknownExport {
    name: Option<String>,
    reason: String,
    guard: SyntaxGuard,
    visibility: ExportVisibility,
}

impl UnknownExport {
    pub(super) fn new(
        name: Option<String>,
        reason: String,
        guard: SyntaxGuard,
        visibility: ExportVisibility,
    ) -> Self {
        Self {
            name,
            reason,
            guard,
            visibility,
        }
    }

    pub(super) fn unbounded(reason: String) -> Self {
        Self::new(
            None,
            reason,
            SyntaxGuard::Ordinary,
            ExportVisibility::default(),
        )
    }

    pub(super) fn matches(&self, name: &str) -> bool {
        self.name
            .as_deref()
            .is_none_or(|candidate| candidate == name)
    }

    pub(super) fn visible_from(&self, consumer: &LogicalModule) -> bool {
        self.visibility.visible_from(consumer)
    }

    pub(super) fn active_for(
        &self,
        consumer: &LogicalModule,
        invocation_guard: &SyntaxGuard,
    ) -> bool {
        self.guard
            .combine(invocation_guard)
            .availability_in_domain(&consumer.domain)
            != GuardAvailability::Absent
    }

    pub(super) fn through_named(&self, edge: &NamedExport) -> Self {
        let mut propagated = self.clone();
        propagated.name = Some(edge.name.clone());
        propagated.guard = propagated.guard.combine(&edge.guard);
        propagated.visibility.restrict(&edge.visibility);
        propagated
    }

    pub(super) fn through_glob(&self, edge: &GlobExport) -> Self {
        let mut propagated = self.clone();
        propagated.guard = propagated.guard.combine(&edge.guard);
        propagated.visibility.restrict(&edge.visibility);
        propagated
    }

    pub(super) fn reason(&self) -> &str {
        &self.reason
    }
}
