//! Ordinary paths retain every namespace identity introduced by include splices.

#[path = "implicit_prelude.rs"]
pub(super) mod implicit_prelude;
#[path = "implicit_prelude_catalog.rs"]
mod implicit_prelude_catalog;
#[path = "include_binding_requirement.rs"]
mod include_binding_requirement;
#[path = "include_prelude.rs"]
mod include_prelude;
#[path = "include_binding_instances.rs"]
mod instance_catalogs;

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::AnalysisQuality;

use super::{
    CompilationIncludeEdge, CompilationModuleEdge, CompilationRoot, ConstructorForm,
    ImportBindingFact, SourceIndex, SourceInstanceId, SourceInstances, SyntaxGuard,
    include_binding_catalog::FileBindings, include_resolution_state::EffectiveModule,
    macro_binding_policy::BindingMacroPolicy,
};

pub(in crate::source) fn known_implicit_prelude_name(name: &str) -> bool {
    let name = name.strip_prefix("r#").unwrap_or(name);
    implicit_prelude_catalog::core(name, "2024").is_some()
        || implicit_prelude_catalog::std_only(name).is_some()
}

pub(super) struct IncludeBindings {
    pub(super) files: BTreeMap<SourceInstanceId, FileBindings>,
    pub(super) inline_module_names:
        BTreeMap<SourceInstanceId, BTreeMap<zrail_core::SourceSpan, String>>,
    pub(super) opaque_namespace_scopes:
        BTreeMap<SourceInstanceId, BTreeSet<(Vec<zrail_core::SourceSpan>, SyntaxGuard, bool)>>,
    pub(super) prelude_directives: BTreeMap<SourceInstanceId, Vec<super::model::PreludeDirective>>,
    pub(super) instances: SourceInstances,
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
    pub(super) origin: ResolvedOrigin,
    pub(super) terminal: ResolvedTerminal,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum ResolvedOrigin {
    CrateLocal,
    External,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum ResolvedTerminal {
    Constructor(ConstructorForm),
    Type,
    Value,
    Module,
    Unknown,
}

impl Default for ResolvedPath {
    fn default() -> Self {
        Self {
            name: String::new(),
            quality: AnalysisQuality::Exact,
            crossed_include: false,
            requires_projection: false,
            blocks_completeness: false,
            origin: ResolvedOrigin::Unknown,
            terminal: ResolvedTerminal::Unknown,
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
        let catalogs = instance_catalogs::collect(index, &instances, binding_macros);
        Self {
            files: catalogs.files,
            inline_module_names: catalogs.inline_module_names,
            opaque_namespace_scopes: catalogs.opaque_namespace_scopes,
            prelude_directives: catalogs.prelude_directives,
            instances,
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

    pub(super) fn active_instances(
        &self,
        file: &str,
        syntax: super::SourceSyntax,
        guard: &SyntaxGuard,
    ) -> Vec<SourceInstanceId> {
        self.instances
            .for_source(file, syntax)
            .iter()
            .copied()
            .filter(|id| {
                self.instances.get(*id).is_some_and(|source| {
                    guard.availability_in_domain(&source.domain).is_available()
                })
            })
            .collect()
    }

    pub(super) fn contextual_projection_is_generic(
        &self,
        file: &str,
        syntax: super::SourceSyntax,
        written: &str,
        guard: &SyntaxGuard,
    ) -> bool {
        let written = written.strip_prefix('<').unwrap_or(written);
        let Some(root) = written.split("::").next() else {
            return false;
        };
        let root = root.strip_prefix("r#").unwrap_or(root);
        self.active_instances(file, syntax, guard)
            .iter()
            .any(|instance| {
                self.instances.get(*instance).is_some_and(|source| {
                    source
                        .generic_types
                        .iter()
                        .any(|generic| generic.strip_prefix("r#").unwrap_or(generic) == root)
                })
            })
    }
}
