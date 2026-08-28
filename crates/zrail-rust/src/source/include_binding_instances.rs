//! Syntax-specific source instances own their exact lexical catalogs.

use std::collections::{BTreeMap, BTreeSet};

use crate::source::{
    BindingKind, ModuleBinding, SourceIndex, SourceInstanceId, SourceInstances, SyntaxGuard,
    include_binding_catalog::FileBindings, macro_binding_policy::BindingMacroPolicy,
    model::PreludeDirective,
};

pub(super) struct InstanceCatalogs {
    pub(super) files: BTreeMap<SourceInstanceId, FileBindings>,
    pub(super) inline_module_names:
        BTreeMap<SourceInstanceId, BTreeMap<zrail_core::SourceSpan, String>>,
    pub(super) opaque_namespace_scopes:
        BTreeMap<SourceInstanceId, BTreeSet<(Vec<zrail_core::SourceSpan>, SyntaxGuard, bool)>>,
    pub(super) prelude_directives: BTreeMap<SourceInstanceId, Vec<PreludeDirective>>,
}

pub(super) fn collect(
    index: &SourceIndex,
    instances: &SourceInstances,
    binding_macros: &BindingMacroPolicy,
) -> InstanceCatalogs {
    let facts = index
        .files
        .iter()
        .map(|file| ((file.relative.as_str(), file.syntax), file))
        .collect::<BTreeMap<_, _>>();
    let mut catalogs = InstanceCatalogs {
        files: BTreeMap::new(),
        inline_module_names: BTreeMap::new(),
        opaque_namespace_scopes: BTreeMap::new(),
        prelude_directives: BTreeMap::new(),
    };
    for (id, source) in instances.iter() {
        let Some(file) = facts.get(&(source.file.as_str(), source.syntax)) else {
            continue;
        };
        catalogs
            .files
            .insert(id, FileBindings::collect(&file.import_bindings));
        catalogs.inline_module_names.insert(
            id,
            file.import_bindings
                .iter()
                .filter_map(|binding| match binding.kind {
                    BindingKind::Module(ModuleBinding::Inline(span)) => {
                        binding.name.as_ref().map(|name| (span, name.clone()))
                    }
                    _ => None,
                })
                .collect(),
        );
        catalogs.opaque_namespace_scopes.insert(
            id,
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
        );
        catalogs
            .prelude_directives
            .insert(id, file.prelude_directives.clone());
    }
    catalogs
}
