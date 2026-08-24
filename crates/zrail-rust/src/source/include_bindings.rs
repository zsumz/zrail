//! Ordinary paths retain every namespace identity introduced by include splices.

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::AnalysisQuality;

use super::{
    CompilationIncludeEdge, CompilationModuleEdge, CompilationRoot, ImportBindingFact,
    ModuleBinding, SourceIndex, SourceInstanceId, SourceInstances, SyntaxGuard,
    include_binding_catalog::FileBindings,
    include_projection_budget::{ProjectionBudget, ProjectionLimit},
    include_resolution_state::EffectiveModule,
    macro_binding_policy::BindingMacroPolicy,
};

pub(super) struct IncludeBindings {
    pub(super) files: BTreeMap<String, FileBindings>,
    pub(super) inline_module_names: BTreeMap<String, BTreeMap<zrail_core::SourceSpan, String>>,
    pub(super) opaque_namespace_scopes:
        BTreeMap<String, BTreeSet<(Vec<zrail_core::SourceSpan>, SyntaxGuard)>>,
    pub(super) instances: SourceInstances,
}

#[derive(Clone)]
pub(super) struct BindingSite {
    pub(super) binding: ImportBindingFact,
    pub(super) instance: SourceInstanceId,
    pub(super) module: EffectiveModule,
    pub(super) crossed_include: bool,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct ResolvedPath {
    pub(super) name: String,
    pub(super) quality: AnalysisQuality,
    pub(super) crossed_include: bool,
    pub(super) requires_projection: bool,
}

impl Default for ResolvedPath {
    fn default() -> Self {
        Self {
            name: String::new(),
            quality: AnalysisQuality::Exact,
            crossed_include: false,
            requires_projection: false,
        }
    }
}

impl IncludeBindings {
    pub(super) fn collect(
        index: &SourceIndex,
        roots: &[CompilationRoot],
        modules: &[CompilationModuleEdge],
        includes: &[CompilationIncludeEdge],
        binding_macros: &BindingMacroPolicy,
    ) -> Self {
        Self {
            files: index
                .files
                .iter()
                .map(|file| {
                    (
                        file.relative.clone(),
                        FileBindings::collect(&file.import_bindings),
                    )
                })
                .collect(),
            inline_module_names: index
                .files
                .iter()
                .map(|file| {
                    (
                        file.relative.clone(),
                        file.import_bindings
                            .iter()
                            .filter_map(|binding| match binding.kind {
                                super::BindingKind::Module(ModuleBinding::Inline(span)) => {
                                    binding.name.as_ref().map(|name| (span, name.clone()))
                                }
                                _ => None,
                            })
                            .collect(),
                    )
                })
                .collect(),
            opaque_namespace_scopes: index
                .files
                .iter()
                .map(|file| {
                    (
                        file.relative.clone(),
                        file.item_macros
                            .iter()
                            .chain(&file.opaque_binding_macros)
                            .filter(|fact| binding_macros.retains_opacity(&file.relative, fact))
                            .map(|fact| (fact.lexical_scope.clone(), fact.guard))
                            .collect(),
                    )
                })
                .collect(),
            instances: SourceInstances::build(roots, modules, includes),
        }
    }

    pub(super) fn active_instances(
        &self,
        file: &str,
        guard: SyntaxGuard,
        budget: &mut ProjectionBudget,
    ) -> Result<Vec<SourceInstanceId>, ProjectionLimit> {
        let mut active = Vec::new();
        for id in self.instances.for_file(file) {
            budget.consume_work()?;
            if self.instances.get(*id).is_some_and(|instance| {
                guard
                    .availability_in(SyntaxGuard::for_test_only(
                        instance.domain.mode.enables_cfg_test(),
                    ))
                    .is_available()
            }) {
                active.push(*id);
            }
        }
        Ok(active)
    }
}
