//! Ordinary paths retain every namespace identity introduced by include splices.

#[path = "include_binding_activity.rs"]
mod include_binding_activity;

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
        BTreeMap<String, BTreeSet<(Vec<zrail_core::SourceSpan>, SyntaxGuard, bool)>>,
    pub(super) instances: SourceInstances,
    active_instances: BTreeMap<(String, SyntaxGuard), Vec<SourceInstanceId>>,
    extern_roots: BTreeMap<String, BTreeSet<String>>,
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
    pub(super) blocks_completeness: bool,
}

impl Default for ResolvedPath {
    fn default() -> Self {
        Self {
            name: String::new(),
            quality: AnalysisQuality::Exact,
            crossed_include: false,
            requires_projection: false,
            blocks_completeness: false,
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

    #[cfg(test)]
    pub(super) fn collect_with_limit(
        index: &SourceIndex,
        roots: &[CompilationRoot],
        modules: &[CompilationModuleEdge],
        includes: &[CompilationIncludeEdge],
        binding_macros: &BindingMacroPolicy,
        derived_limit: Option<usize>,
    ) -> Self {
        Self::collect_with_extern_roots(
            index,
            roots,
            modules,
            includes,
            binding_macros,
            derived_limit,
            BTreeMap::new(),
        )
    }

    pub(super) fn collect_with_extern_roots(
        index: &SourceIndex,
        roots: &[CompilationRoot],
        modules: &[CompilationModuleEdge],
        includes: &[CompilationIncludeEdge],
        binding_macros: &BindingMacroPolicy,
        derived_limit: Option<usize>,
        extern_roots: BTreeMap<String, BTreeSet<String>>,
    ) -> Self {
        let instances = SourceInstances::build_with_limit(roots, modules, includes, derived_limit);
        let active_instances = include_binding_activity::active_instances(index, &instances);
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
                            .map(|fact| {
                                (
                                    fact.lexical_scope.clone(),
                                    fact.guard.clone(),
                                    binding_macros.opacity_is_authorized(&file.relative, fact),
                                )
                            })
                            .collect(),
                    )
                })
                .collect(),
            instances,
            active_instances,
            extern_roots,
        }
    }

    pub(super) fn is_extern_root(&self, instance: SourceInstanceId, root: &str) -> bool {
        self.instances.get(instance).is_some_and(|source| {
            self.extern_roots
                .get(&source.domain.package)
                .is_some_and(|roots| roots.contains(root.strip_prefix("r#").unwrap_or(root)))
        })
    }

    pub(super) fn active_instances(&self, file: &str, guard: &SyntaxGuard) -> &[SourceInstanceId] {
        self.active_instances
            .get(&(file.to_owned(), guard.clone()))
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

fn written_root(path: &str) -> Option<&str> {
    let root = path.trim_start_matches("::").split("::").next()?;
    let root = root.strip_prefix("r#").unwrap_or(root);
    (!root.is_empty()).then_some(root)
}

fn is_qualifier(root: &str) -> bool {
    root.starts_with('<') || matches!(root, "crate" | "self" | "super" | "Self")
}
