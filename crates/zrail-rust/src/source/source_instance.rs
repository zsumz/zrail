//! Cargo roots, modules, and include occurrences form exact source instances.

use std::collections::{BTreeMap, VecDeque};

use super::{CompilationDomain, CompilationIncludeEdge, CompilationModuleEdge, SyntaxGuard};

const MAX_SOURCE_INSTANCES: usize = 4096;
const MAX_SOURCE_INSTANCE_DEPTH: usize = 128;

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
    pub(crate) parent: Option<SourceInstanceId>,
    pub(crate) entered_from: SourceEntry,
    depth: usize,
}

pub(crate) struct SourceInstances {
    instances: Vec<SourceInstance>,
    by_file: BTreeMap<String, Vec<SourceInstanceId>>,
    module_children: BTreeMap<SourceInstanceId, Vec<(CompilationModuleEdge, SourceInstanceId)>>,
    include_children: BTreeMap<SourceInstanceId, Vec<(CompilationIncludeEdge, SourceInstanceId)>>,
    pub(crate) complete: bool,
}

impl SourceInstances {
    pub(crate) fn build(
        roots: &[CompilationRoot],
        modules: &[CompilationModuleEdge],
        includes: &[CompilationIncludeEdge],
    ) -> Self {
        let module_edges = grouped_modules(modules);
        let include_edges = grouped_includes(includes);
        let mut graph = Self {
            instances: Vec::new(),
            by_file: BTreeMap::new(),
            module_children: BTreeMap::new(),
            include_children: BTreeMap::new(),
            complete: true,
        };
        let mut queue = VecDeque::new();
        for root in roots {
            if let Some(id) = graph.push(
                root.file.clone(),
                root.domain.clone(),
                None,
                SourceEntry::CargoRoot,
                SyntaxGuard::Ordinary,
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

    pub(crate) fn get(&self, id: SourceInstanceId) -> Option<&SourceInstance> {
        self.instances.get(id.0)
    }

    pub(crate) fn for_file(&self, file: &str) -> &[SourceInstanceId] {
        self.by_file.get(file).map_or(&[], Vec::as_slice)
    }

    pub(crate) fn includes_from(
        &self,
        parent: SourceInstanceId,
    ) -> &[(CompilationIncludeEdge, SourceInstanceId)] {
        self.include_children
            .get(&parent)
            .map_or(&[], Vec::as_slice)
    }

    pub(crate) fn modules_from(
        &self,
        parent: SourceInstanceId,
    ) -> &[(CompilationModuleEdge, SourceInstanceId)] {
        self.module_children.get(&parent).map_or(&[], Vec::as_slice)
    }

    fn add_child(
        &mut self,
        parent: SourceInstanceId,
        file: String,
        entry: SourceEntry,
    ) -> Option<SourceInstanceId> {
        let parent_instance = &self.instances[parent.0];
        let depth = parent_instance.depth + 1;
        if depth > MAX_SOURCE_INSTANCE_DEPTH || self.ancestor_contains(parent, &file) {
            self.complete = false;
            return None;
        }
        let guard = match &entry {
            SourceEntry::Module(edge) => edge.guard,
            SourceEntry::Include(edge) => edge.guard,
            SourceEntry::CargoRoot => return None,
        };
        self.push(
            file,
            parent_instance.domain.clone(),
            Some(parent),
            entry,
            guard,
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
        depth: usize,
    ) -> Option<SourceInstanceId> {
        if self.instances.len() >= MAX_SOURCE_INSTANCES {
            self.complete = false;
            return None;
        }
        let id = SourceInstanceId(self.instances.len());
        self.by_file.entry(file.clone()).or_default().push(id);
        self.instances.push(SourceInstance {
            file,
            domain,
            guard,
            parent,
            entered_from,
            depth,
        });
        Some(id)
    }

    fn ancestor_contains(&self, mut parent: SourceInstanceId, file: &str) -> bool {
        loop {
            let instance = &self.instances[parent.0];
            if instance.file == file {
                return true;
            }
            let Some(next) = instance.parent else {
                return false;
            };
            parent = next;
        }
    }
}

fn grouped_modules(
    edges: &[CompilationModuleEdge],
) -> BTreeMap<(String, CompilationDomain), Vec<&CompilationModuleEdge>> {
    let mut grouped = BTreeMap::new();
    for edge in edges {
        grouped
            .entry((edge.parent.clone(), edge.domain.clone()))
            .or_insert_with(Vec::new)
            .push(edge);
    }
    grouped
}

fn grouped_includes(
    edges: &[CompilationIncludeEdge],
) -> BTreeMap<(String, CompilationDomain), Vec<&CompilationIncludeEdge>> {
    let mut grouped = BTreeMap::new();
    for edge in edges {
        grouped
            .entry((edge.parent.clone(), edge.domain.clone()))
            .or_insert_with(Vec::new)
            .push(edge);
    }
    grouped
}
