//! Bounded breadth-first construction retains every exact source occurrence.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::source::{
    CompilationIncludeEdge, CompilationModuleEdge, SyntaxGuard,
    source_instance_edges::{
        MIN_DERIVED_SOURCE_CONTEXTS, SourceInstanceMetrics, grouped_includes, grouped_modules,
    },
};

use super::{
    CompilationRoot, SourceEntry, SourceInstances, SourceOccurrence, inheritance::InheritedBindings,
};

impl SourceInstances {
    #[cfg(test)]
    pub(crate) fn build(
        roots: &[CompilationRoot],
        modules: &[CompilationModuleEdge],
        includes: &[CompilationIncludeEdge],
    ) -> Self {
        Self::build_with_limit(roots, modules, includes, None)
    }

    pub(crate) fn build_with_limit(
        roots: &[CompilationRoot],
        modules: &[CompilationModuleEdge],
        includes: &[CompilationIncludeEdge],
        derived_limit: Option<usize>,
    ) -> Self {
        let module_edges = grouped_modules(modules);
        let include_edges = grouped_includes(includes);
        let derived_limit = derived_limit.unwrap_or_else(|| {
            modules
                .len()
                .saturating_add(includes.len())
                .saturating_mul(8)
                .max(MIN_DERIVED_SOURCE_CONTEXTS)
        });
        let mut graph = Self {
            instances: Vec::new(),
            by_file: BTreeMap::new(),
            module_children: BTreeMap::new(),
            include_children: BTreeMap::new(),
            identities: BTreeSet::new(),
            derived_limit,
            issues: Vec::new(),
            metrics: SourceInstanceMetrics::default(),
        };
        let mut queue = VecDeque::new();
        for root in roots {
            if let Some(id) = graph.push(
                SourceOccurrence {
                    file: root.file.clone(),
                    syntax: root.syntax,
                },
                root.domain.clone(),
                None,
                SourceEntry::CargoRoot,
                SyntaxGuard::Ordinary,
                InheritedBindings::default(),
                0,
            ) {
                queue.push_back(id);
            }
        }
        while let Some(parent) = queue.pop_front() {
            let instance = graph.instances[parent.0].clone();
            let key = (
                instance.file.clone(),
                instance.syntax,
                instance.domain.clone(),
            );
            for edge in module_edges.get(&key).into_iter().flatten() {
                if let Some(child) = graph.add_child(
                    parent,
                    edge.child.clone(),
                    SourceEntry::Module((*edge).clone()),
                ) {
                    graph
                        .module_children
                        .entry(parent)
                        .or_default()
                        .push(((*edge).clone(), child));
                    queue.push_back(child);
                }
            }
            for edge in include_edges.get(&key).into_iter().flatten() {
                if let Some(child) = graph.add_child(
                    parent,
                    edge.child.clone(),
                    SourceEntry::Include((*edge).clone()),
                ) {
                    graph
                        .include_children
                        .entry(parent)
                        .or_default()
                        .push(((*edge).clone(), child));
                    queue.push_back(child);
                }
            }
        }
        graph
    }
}
