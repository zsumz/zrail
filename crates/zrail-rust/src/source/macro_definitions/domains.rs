//! Macro definition lookup is restricted to active compilation domains.

use super::{MacroDefinitions, SourceInstanceId, SourceSyntax, SyntaxGuard};

impl MacroDefinitions {
    pub(super) fn active_instances(
        &self,
        file: &str,
        syntax: SourceSyntax,
        guard: &SyntaxGuard,
    ) -> Option<Vec<SourceInstanceId>> {
        if !self.instances.is_complete() {
            return None;
        }
        Some(
            self.instances
                .for_source(file, syntax)
                .iter()
                .copied()
                .filter(|id| {
                    self.instances.get(*id).is_some_and(|instance| {
                        guard
                            .availability_in_domain(&instance.domain)
                            .is_available()
                    })
                })
                .collect(),
        )
    }
}
