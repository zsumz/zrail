//! Macro definition lookup is restricted to active compilation domains.

use super::{CompilationDomain, MacroDefinitions, SourceInstanceId, SyntaxGuard};

impl MacroDefinitions {
    pub(super) fn active_instances(
        &self,
        file: &str,
        guard: SyntaxGuard,
    ) -> Option<Vec<SourceInstanceId>> {
        if !self.instances.is_complete() {
            return None;
        }
        Some(
            self.instances
                .for_file(file)
                .iter()
                .copied()
                .filter(|id| {
                    self.instances
                        .get(*id)
                        .is_some_and(|instance| guard.available_in(domain_guard(&instance.domain)))
                })
                .collect(),
        )
    }

    pub(super) fn active_domains(
        &self,
        file: &str,
        guard: SyntaxGuard,
    ) -> Option<Vec<&CompilationDomain>> {
        let domains = self.domains.get(file)?;
        Some(
            domains
                .iter()
                .filter(|domain| guard.available_in(domain_guard(domain)))
                .collect(),
        )
    }
}

fn domain_guard(domain: &CompilationDomain) -> SyntaxGuard {
    SyntaxGuard::for_test_only(domain.mode.enables_cfg_test())
}
