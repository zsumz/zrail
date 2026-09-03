//! Physical facts are mounted into logical module export drafts.

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::AnalysisQuality;

use super::super::{
    BindingKind, CompilationMode, MacroDefinitionExport, SourceIndex, SourceInstances,
    logical_modules::{inline_catalog, locate},
};
use super::{ExportVisibility, ExportedMacro, MacroExportSet, MacroExports, ModuleDraft};

impl MacroExports {
    pub(in crate::source) fn collect(
        index: &SourceIndex,
        cargo: &crate::cargo::CargoWorkspace,
        resolved_cargo: Option<&crate::cargo::ResolvedCargoGraph>,
        instances: &SourceInstances,
    ) -> Self {
        let inline = inline_catalog(index);
        let files = index
            .files
            .iter()
            .map(|file| ((file.relative.as_str(), file.syntax), file))
            .collect::<BTreeMap<_, _>>();
        let package_directories = cargo
            .packages
            .iter()
            .map(|package| (package.name.clone(), package.directory.clone()))
            .collect();
        let package_dependencies = cargo
            .packages
            .iter()
            .map(|package| (package.name.clone(), package.dependencies.clone()))
            .collect();
        let mut drafts = BTreeMap::<_, ModuleDraft>::new();
        let mut contexts = BTreeMap::<_, BTreeSet<_>>::new();
        let mut modules = BTreeSet::new();
        let mut package_roots = BTreeMap::<String, BTreeSet<_>>::new();
        for (instance, mount) in instances.iter() {
            let Some(file) = files.get(&(mount.file.as_str(), mount.syntax)) else {
                continue;
            };
            let Some(base) = locate(instances, instance, &[], &inline) else {
                continue;
            };
            modules.insert(base.clone());
            drafts.entry(base.clone()).or_default();
            if base.path.is_empty()
                && matches!(
                    base.domain.mode,
                    CompilationMode::Library | CompilationMode::ProcMacro
                )
            {
                package_roots
                    .entry(base.domain.package.clone())
                    .or_default()
                    .insert(base.clone());
            }
            collect_modules(
                file,
                instances,
                instance,
                &inline,
                &mut modules,
                &mut drafts,
            );
            collect_definitions(
                file,
                instances,
                instance,
                mount,
                &inline,
                &package_directories,
                &mut drafts,
            );
            super::imports::collect(file, instances, instance, mount, &inline, &mut drafts);
            super::contexts::collect(
                file,
                &files,
                instances,
                instance,
                mount,
                &inline,
                &mut contexts,
            );
        }
        let external =
            super::external::ExternalMacroCatalog::collect(cargo, resolved_cargo, &drafts);
        let mut exports = Self {
            sets: drafts
                .keys()
                .cloned()
                .map(|module| (module, MacroExportSet::default()))
                .collect(),
            contexts,
            modules,
            package_directories,
            package_dependencies,
            package_roots,
            external,
        };
        exports.close(drafts);
        exports
    }
}

fn collect_modules(
    file: &super::super::RustFileFacts,
    instances: &SourceInstances,
    instance: super::super::SourceInstanceId,
    inline: &super::super::logical_modules::InlineModuleCatalog,
    modules: &mut BTreeSet<super::super::logical_modules::LogicalModule>,
    drafts: &mut BTreeMap<super::super::logical_modules::LogicalModule, ModuleDraft>,
) {
    let Some(source) = instances.get(instance) else {
        return;
    };
    for binding in &file.import_bindings {
        if !matches!(binding.kind, BindingKind::Module(_)) {
            continue;
        }
        if !source
            .guard
            .combine(&binding.guard)
            .availability_in_domain(&source.domain)
            .is_available()
        {
            continue;
        }
        let Some(name) = binding.name.as_deref() else {
            continue;
        };
        let Some(parent) = locate(instances, instance, &binding.lexical_scope, inline) else {
            continue;
        };
        let module = parent.child(name);
        modules.insert(module.clone());
        drafts.entry(module).or_default();
    }
}

fn collect_definitions(
    file: &super::super::RustFileFacts,
    instances: &SourceInstances,
    instance: super::super::SourceInstanceId,
    mount: &super::super::source_instance::SourceInstance,
    inline: &super::super::logical_modules::InlineModuleCatalog,
    package_directories: &BTreeMap<String, String>,
    drafts: &mut BTreeMap<super::super::logical_modules::LogicalModule, ModuleDraft>,
) {
    for definition in &file.macro_definitions {
        let Some(module) = locate(instances, instance, &definition.lexical_scope, inline) else {
            continue;
        };
        let Some(directory) = package_directories.get(&mount.domain.package) else {
            continue;
        };
        let exported = ExportedMacro {
            origins: vec![super::super::MacroOrigin::Repository {
                package: mount.domain.package.clone(),
                directory: directory.clone(),
            }],
            proc_macro: definition.export == MacroDefinitionExport::ProcMacro,
            authority_name: None,
            definition: Some(file.relative.clone()),
            definition_name: Some(definition.name.clone()),
            definition_sha256: Some(definition.sha256.clone()),
            guard: mount.guard.combine(&definition.guard),
            quality: AnalysisQuality::Exact,
            visibility: ExportVisibility::private(&module),
        };
        drafts
            .entry(module.clone())
            .or_default()
            .local
            .entry(definition.name.clone())
            .or_default()
            .insert(exported.clone());
        let root_export = match definition.export {
            MacroDefinitionExport::CrateRoot => true,
            MacroDefinitionExport::ProcMacro => matches!(
                mount.domain.mode,
                CompilationMode::ProcMacro | CompilationMode::ProcMacroTest
            ),
            MacroDefinitionExport::Lexical => false,
        };
        if root_export {
            let root = module.root();
            let mut exported = exported;
            exported.visibility = ExportVisibility::default();
            drafts
                .entry(root)
                .or_default()
                .direct
                .entry(definition.name.clone())
                .or_default()
                .insert(exported);
        }
    }
}
