//! Ordinary paths retain every namespace identity introduced by include splices.

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::AnalysisQuality;

use super::{
    CompilationIncludeEdge, CompilationModuleEdge, CompilationRoot, ImportBindingFact,
    ModuleBinding, RustFileFacts, SourceIndex, SourceInstanceId, SourceInstances, SyntaxGuard,
    include_binding_catalog::FileBindings, include_resolution_state::EffectiveModule,
    macro_binding_policy::BindingMacroPolicy,
};

pub(super) struct IncludeBindings {
    pub(super) files: BTreeMap<String, FileBindings>,
    pub(super) inline_module_names: BTreeMap<String, BTreeMap<zrail_core::SourceSpan, String>>,
    pub(super) opaque_namespace_scopes:
        BTreeMap<String, BTreeSet<(Vec<zrail_core::SourceSpan>, SyntaxGuard)>>,
    pub(super) instances: SourceInstances,
    active_instances: BTreeMap<(String, SyntaxGuard), Vec<SourceInstanceId>>,
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
    #[cfg(test)]
    pub(super) fn collect(
        index: &SourceIndex,
        roots: &[CompilationRoot],
        modules: &[CompilationModuleEdge],
        includes: &[CompilationIncludeEdge],
        binding_macros: &BindingMacroPolicy,
    ) -> Self {
        Self::collect_with_limit(index, roots, modules, includes, binding_macros, None)
    }

    pub(super) fn collect_with_limit(
        index: &SourceIndex,
        roots: &[CompilationRoot],
        modules: &[CompilationModuleEdge],
        includes: &[CompilationIncludeEdge],
        binding_macros: &BindingMacroPolicy,
        derived_limit: Option<usize>,
    ) -> Self {
        let instances = SourceInstances::build_with_limit(roots, modules, includes, derived_limit);
        let active_instances = active_instances(index, &instances);
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
            instances,
            active_instances,
        }
    }

    pub(super) fn active_instances(&self, file: &str, guard: SyntaxGuard) -> &[SourceInstanceId] {
        self.active_instances
            .get(&(file.to_owned(), guard))
            .map_or(&[], Vec::as_slice)
    }

    pub(super) fn requires_ordinary_resolution(&self, file: &RustFileFacts) -> bool {
        if file.paths.iter().chain(&file.calls).any(|fact| {
            fact.quality == AnalysisQuality::Unresolved
                && fact
                    .written
                    .as_deref()
                    .is_some_and(|written| !written.trim_start_matches("::").contains("::"))
        }) {
            return true;
        }
        let roots = file
            .paths
            .iter()
            .chain(&file.calls)
            .filter_map(|fact| fact.written.as_deref())
            .filter_map(written_root)
            .collect::<BTreeSet<_>>();
        self.instances.for_file(&file.relative).iter().any(|id| {
            roots
                .iter()
                .any(|root| is_qualifier(root) || self.ancestor_can_bind(*id, root))
        })
    }

    fn ancestor_can_bind(&self, mut id: SourceInstanceId, root: &str) -> bool {
        loop {
            let Some(instance) = self.instances.get(id) else {
                return false;
            };
            if self.files.get(&instance.file).is_some_and(|bindings| {
                bindings.named.contains_key(root) || !bindings.globs.is_empty()
            }) || self
                .opaque_namespace_scopes
                .get(&instance.file)
                .is_some_and(|scopes| !scopes.is_empty())
            {
                return true;
            }
            let Some(parent) = instance.parent else {
                return false;
            };
            id = parent;
        }
    }
}

fn active_instances(
    index: &SourceIndex,
    instances: &SourceInstances,
) -> BTreeMap<(String, SyntaxGuard), Vec<SourceInstanceId>> {
    index
        .files
        .iter()
        .flat_map(|file| {
            file.paths
                .iter()
                .chain(&file.calls)
                .filter(|fact| fact.written.is_some())
                .map(|fact| (file.relative.clone(), fact.guard))
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(|(file, guard)| {
            let active = instances
                .for_file(&file)
                .iter()
                .copied()
                .filter(|id| {
                    instances.get(*id).is_some_and(|instance| {
                        guard.available_in(SyntaxGuard::for_test_only(
                            instance.domain.mode.enables_cfg_test(),
                        ))
                    })
                })
                .collect();
            ((file, guard), active)
        })
        .collect()
}

fn written_root(path: &str) -> Option<&str> {
    let root = path.trim_start_matches("::").split("::").next()?;
    let root = root.strip_prefix("r#").unwrap_or(root);
    (!root.is_empty()).then_some(root)
}

fn is_qualifier(root: &str) -> bool {
    root.starts_with('<') || matches!(root, "crate" | "self" | "super" | "Self")
}
