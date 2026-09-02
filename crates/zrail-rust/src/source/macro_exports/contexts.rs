//! Invocation contexts retain every logical mount of one physical syntax site.

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::SourceSpan;

use super::super::{
    BindingKind, IncludeContext, MacroDerivation, RustFileFacts, SourceEntry, SourceInstanceId,
    SourceInstances, SourceSyntax, SyntaxGuard,
    logical_modules::{InlineModuleCatalog, locate},
    source_instance::SourceInstance,
};
use super::{MountedImport, MountedModule};

pub(super) fn collect(
    file: &RustFileFacts,
    files: &BTreeMap<(&str, SourceSyntax), &RustFileFacts>,
    instances: &SourceInstances,
    instance: SourceInstanceId,
    mount: &SourceInstance,
    inline: &InlineModuleCatalog,
    contexts: &mut BTreeMap<(String, SourceSyntax, Vec<SourceSpan>), BTreeSet<MountedModule>>,
) {
    for expansion in file
        .macro_expansions
        .iter()
        .chain(&file.opaque_macro_inputs)
        .chain(file.compile_effects.iter().map(|effect| &effect.invocation))
    {
        let Some(module) = locate(instances, instance, &expansion.lexical_scope, inline) else {
            continue;
        };
        contexts
            .entry((
                file.relative.clone(),
                file.syntax,
                expansion.lexical_scope.clone(),
            ))
            .or_default()
            .insert(MountedModule {
                instance,
                module,
                guard: mount.guard.clone(),
                inherited_imports: mounted_imports(
                    instances,
                    files,
                    instance,
                    &expansion.lexical_scope,
                    &expansion.name,
                ),
            });
    }
}

fn mounted_imports(
    instances: &SourceInstances,
    files: &BTreeMap<(&str, SourceSyntax), &RustFileFacts>,
    mut instance: SourceInstanceId,
    scope: &[SourceSpan],
    written: &str,
) -> BTreeSet<MountedImport> {
    let mut imports = BTreeSet::new();
    collect_included(
        instances,
        files,
        instance,
        scope,
        written,
        &SyntaxGuard::Ordinary,
        &mut imports,
    );
    while let Some(source) = instances.get(instance) {
        let (parent, scope) = match (&source.parent, &source.entered_from) {
            (Some(parent), SourceEntry::Include(edge)) => (*parent, edge.parent_scope.as_slice()),
            _ => break,
        };
        let Some(parent_source) = instances.get(parent) else {
            break;
        };
        if let Some(file) = files.get(&(parent_source.file.as_str(), parent_source.syntax)) {
            collect_visible(file, scope, written, &SyntaxGuard::Ordinary, &mut imports);
        }
        collect_included(
            instances,
            files,
            parent,
            scope,
            written,
            &SyntaxGuard::Ordinary,
            &mut imports,
        );
        instance = parent;
    }
    imports
}

fn collect_included(
    instances: &SourceInstances,
    files: &BTreeMap<(&str, SourceSyntax), &RustFileFacts>,
    instance: SourceInstanceId,
    scope: &[SourceSpan],
    written: &str,
    guard: &SyntaxGuard,
    imports: &mut BTreeSet<MountedImport>,
) {
    for (edge, child) in instances.includes_from(instance) {
        if edge.context != IncludeContext::Items || !scope.starts_with(&edge.parent_scope) {
            continue;
        }
        let child_guard = guard.combine(&edge.guard);
        let Some(child_source) = instances.get(*child) else {
            continue;
        };
        if let Some(file) = files.get(&(child_source.file.as_str(), child_source.syntax)) {
            collect_visible(file, &[], written, &child_guard, imports);
        }
        collect_included(
            instances,
            files,
            *child,
            &[],
            written,
            &child_guard,
            imports,
        );
    }
}

fn collect_visible(
    file: &RustFileFacts,
    scope: &[SourceSpan],
    written: &str,
    mount_guard: &SyntaxGuard,
    imports: &mut BTreeSet<MountedImport>,
) {
    let (root, suffix) = split_root(written);
    for binding in &file.import_bindings {
        if !scope.starts_with(&binding.lexical_scope) {
            continue;
        }
        let (target, derivation) = match binding.kind {
            BindingKind::Import if binding.name.as_deref() == Some(root) => (
                format!("{}{suffix}", binding.target),
                MacroDerivation::ExactImport,
            ),
            BindingKind::Glob => (
                format!("{}::{written}", binding.target),
                MacroDerivation::GlobImport,
            ),
            _ => continue,
        };
        imports.insert(MountedImport {
            target,
            derivation,
            guard: mount_guard.combine(&binding.guard),
            quality: binding.quality,
        });
    }
}

fn split_root(path: &str) -> (&str, &str) {
    path.split_once("::")
        .map_or((path, ""), |(root, _)| (root, &path[root.len()..]))
}
