//! Attribute-replaced module mounts remain opaque across exact graph edges and includes.

use std::collections::{BTreeMap, BTreeSet};

use crate::source::{
    BindingKind, CompilationDomain, CompilationIncludeEdge, CompilationModuleEdge, ModuleBinding,
    SourceIndex,
};

pub(crate) fn inherit_replacing_mounts(
    source: &mut SourceIndex,
    modules: &[CompilationModuleEdge],
    includes: &[CompilationIncludeEdge],
) {
    let files = source
        .files
        .iter()
        .map(|file| (&file.relative, file))
        .collect::<BTreeMap<_, _>>();
    let mut pending = BTreeSet::new();
    let mut children = BTreeMap::<(String, CompilationDomain), BTreeSet<String>>::new();
    for edge in modules {
        if !edge
            .guard
            .availability_in_domain(&edge.domain)
            .is_available()
        {
            continue;
        }
        children
            .entry((edge.parent.clone(), edge.domain.clone()))
            .or_default()
            .insert(edge.child.clone());
        let replaced = files.get(&edge.parent).is_some_and(|file| {
            file.import_bindings.iter().any(|binding| {
                // The edge and binding share the exact authored module identifier anchor.
                matches!(binding.kind, BindingKind::Module(ModuleBinding::External(span))
                    if Some(span) == edge.span)
                    && binding.lexical_scope == edge.parent_scope
                    && binding.replacement_macros.iter().any(|occurrence| {
                        occurrence
                            .guard
                            .availability_in_domain(&edge.domain)
                            .is_available()
                    })
            })
        });
        if replaced {
            pending.insert((edge.child.clone(), edge.domain.clone()));
        }
    }
    for edge in includes {
        if edge
            .guard
            .availability_in_domain(&edge.domain)
            .is_available()
        {
            children
                .entry((edge.parent.clone(), edge.domain.clone()))
                .or_default()
                .insert(edge.child.clone());
        }
    }
    let mut affected = BTreeSet::new();
    while let Some(key) = pending.pop_first() {
        if !affected.insert(key.clone()) {
            continue;
        }
        if let Some(paths) = children.get(&key) {
            pending.extend(paths.iter().map(|path| (path.clone(), key.1.clone())));
        }
    }
    for file in &mut source.files {
        let domains = affected
            .iter()
            .filter(|(path, _)| path == &file.relative)
            .map(|(_, domain)| domain.clone())
            .collect::<BTreeSet<_>>();
        for declaration in &mut file.type_policy.declarations {
            declaration.replacing_mounts.clone_from(&domains);
        }
    }
}
