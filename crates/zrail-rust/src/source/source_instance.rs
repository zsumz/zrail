//! Cargo roots, modules, and include occurrences form exact source instances.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use super::{
    CompilationDomain, CompilationIncludeEdge, CompilationModuleEdge, IncludeContext, SyntaxGuard,
};

use super::{
    source_instance_edges::{MIN_DERIVED_SOURCE_CONTEXTS, SourceInstanceMetrics},
    source_instance_edges::{grouped_includes, grouped_modules},
};

const MAX_SOURCE_INSTANCE_DEPTH: usize = 128;

pub(crate) use super::source_instance_edges::SourceInstanceIssue;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct SourceInstanceId(pub(crate) usize);

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct CompilationRoot {
    pub(crate) file: String,
    pub(crate) domain: CompilationDomain,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum SourceEntry {
    CargoRoot,
    Module(CompilationModuleEdge),
    Include(CompilationIncludeEdge),
}

#[derive(Clone, Debug)]
pub(crate) struct SourceInstance {
    pub(crate) file: String,
    pub(crate) domain: CompilationDomain,
    pub(crate) guard: SyntaxGuard,
    pub(crate) generic_types: Vec<String>,
    pub(crate) parent: Option<SourceInstanceId>,
    pub(crate) entered_from: SourceEntry,
    depth: usize,
}

pub(crate) struct SourceInstances {
    instances: Vec<SourceInstance>,
    by_file: BTreeMap<String, Vec<SourceInstanceId>>,
    module_children: BTreeMap<SourceInstanceId, Vec<(CompilationModuleEdge, SourceInstanceId)>>,
    include_children: BTreeMap<SourceInstanceId, Vec<(CompilationIncludeEdge, SourceInstanceId)>>,
    identities: BTreeSet<(String, CompilationDomain)>,
    derived_limit: usize,
    issues: Vec<SourceInstanceIssue>,
    metrics: SourceInstanceMetrics,
}

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
                root.file.clone(),
                root.domain.clone(),
                None,
                SourceEntry::CargoRoot,
                SyntaxGuard::Ordinary,
                Vec::new(),
                0,
            ) {
                queue.push_back(id);
            }
        }
        while let Some(parent) = queue.pop_front() {
            let instance = graph.instances[parent.0].clone();
            let key = (instance.file.clone(), instance.domain.clone());
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

    fn add_child(
        &mut self,
        parent: SourceInstanceId,
        file: String,
        entry: SourceEntry,
    ) -> Option<SourceInstanceId> {
        let parent_instance = &self.instances[parent.0];
        let depth = parent_instance.depth + 1;
        if depth > MAX_SOURCE_INSTANCE_DEPTH {
            let mut chain = self.ancestor_chain(parent);
            chain.push(file.clone());
            self.record_issue(SourceInstanceIssue::DepthLimit { file, depth, chain });
            return None;
        }
        if self.ancestor_contains(parent, &file) {
            let mut chain = self.ancestor_chain(parent);
            chain.push(file);
            self.record_issue(SourceInstanceIssue::Cycle { chain });
            return None;
        }
        let guard = match &entry {
            SourceEntry::Module(edge) => edge.guard.clone(),
            SourceEntry::Include(edge) => edge.guard.clone(),
            SourceEntry::CargoRoot => return None,
        };
        let generic_types = match &entry {
            SourceEntry::Include(edge) if edge.context == IncludeContext::Expression => {
                let mut generic_types = parent_instance.generic_types.clone();
                generic_types.extend(edge.generic_types.iter().cloned());
                generic_types.sort();
                generic_types.dedup();
                generic_types
            }
            _ => Vec::new(),
        };
        self.push(
            file,
            parent_instance.domain.clone(),
            Some(parent),
            entry,
            guard,
            generic_types,
            depth,
        )
    }

    fn push(
        &mut self,
        file: String,
        domain: CompilationDomain,
        parent: Option<SourceInstanceId>,
        entered_from: SourceEntry,
        guard: SyntaxGuard,
        generic_types: Vec<String>,
        depth: usize,
    ) -> Option<SourceInstanceId> {
        let identity = (file.clone(), domain.clone());
        let base = self.identities.insert(identity);
        if !base && self.metrics.derived_contexts >= self.derived_limit {
            self.record_issue(SourceInstanceIssue::DerivedContextLimit {
                used: self.metrics.derived_contexts.saturating_add(1),
                limit: self.derived_limit,
                file,
            });
            return None;
        }
        if base {
            self.metrics.base_contexts = self.metrics.base_contexts.saturating_add(1);
        } else {
            self.metrics.derived_contexts = self.metrics.derived_contexts.saturating_add(1);
        }
        let id = SourceInstanceId(self.instances.len());
        self.by_file.entry(file.clone()).or_default().push(id);
        self.instances.push(SourceInstance {
            file,
            domain,
            guard,
            generic_types,
            parent,
            entered_from,
            depth,
        });
        Some(id)
    }
}

#[path = "source_instance_access.rs"]
mod access;

#[cfg(test)]
#[path = "source_instance_test.rs"]
mod source_instance_test;
