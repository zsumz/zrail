//! Exact external-module edges expose only imports reachable through Rust module paths.

use std::collections::{BTreeMap, BTreeSet};

use super::{MacroImportFact, ResolvedModuleEdge, SourceIndex};

const MAX_EDGES_PER_MODULE: usize = 4;
const MAX_IMPORTS_PER_NAME: usize = 4;

#[derive(Default)]
pub(super) struct MacroVisibility {
    imports: BTreeMap<(String, String), Vec<MacroImportFact>>,
    import_overflow: BTreeSet<(String, String)>,
    children: BTreeMap<(String, String), Vec<String>>,
    child_overflow: BTreeSet<(String, String)>,
    parents: BTreeMap<String, Vec<String>>,
    parent_overflow: BTreeSet<String>,
}

pub(super) enum VisibilityLookup<'a> {
    Known(Vec<&'a MacroImportFact>),
    Unknown,
}

impl MacroVisibility {
    pub(super) fn collect(index: &SourceIndex, module_edges: &[ResolvedModuleEdge]) -> Self {
        let mut visibility = Self::default();
        for file in &index.files {
            for import in &file.macro_imports {
                visibility.insert_import(&file.relative, import);
            }
        }
        for edge in module_edges {
            visibility.insert_edge(&edge.parent, &edge.module_name, &edge.child);
        }
        visibility
    }

    pub(super) fn imports_for<'a>(&'a self, file: &str, path: &str) -> VisibilityLookup<'a> {
        let segments = path.split("::").collect::<Vec<_>>();
        let Some((name, prefix)) = segments.split_last() else {
            return VisibilityLookup::Unknown;
        };
        let root = prefix.first().copied();
        let Some(mut nodes) = self.start_nodes(file, root) else {
            return VisibilityLookup::Unknown;
        };
        let mut module_index = 1;
        while root == Some("super") && prefix.get(module_index) == Some(&"super") {
            let Some(parents) = self.parents_of(&nodes) else {
                return VisibilityLookup::Unknown;
            };
            nodes = parents;
            module_index += 1;
        }
        for module in prefix.iter().skip(module_index) {
            let Some(next) = self.child_nodes(&nodes, module) else {
                return VisibilityLookup::Unknown;
            };
            if next.is_empty() {
                return VisibilityLookup::Known(Vec::new());
            }
            nodes = next;
        }
        let mut imports = Vec::new();
        for node in nodes {
            let key = (node, (*name).to_owned());
            if self.import_overflow.contains(&key) {
                return VisibilityLookup::Unknown;
            }
            imports.extend(self.imports.get(&key).into_iter().flatten());
        }
        VisibilityLookup::Known(imports)
    }

    fn start_nodes(&self, file: &str, root: Option<&str>) -> Option<Vec<String>> {
        match root {
            Some("self") => Some(vec![file.to_owned()]),
            Some("super") => self.parent_nodes_or_inline(file),
            Some("crate") => self.root_nodes(file),
            Some(module) => self.child_nodes(&[file.to_owned()], module),
            None => None,
        }
    }

    pub(super) fn repository_candidate(&self, file: &str, path: &str) -> bool {
        if super::macro_visibility::repository_path(path) {
            return true;
        }
        let Some(root) = path.split("::").next() else {
            return false;
        };
        let key = (file.to_owned(), root.to_owned());
        self.children.contains_key(&key) || self.child_overflow.contains(&key)
    }

    fn child_nodes(&self, nodes: &[String], module: &str) -> Option<Vec<String>> {
        let mut children = Vec::new();
        for node in nodes {
            let key = (node.clone(), module.to_owned());
            if self.child_overflow.contains(&key) {
                return None;
            }
            children.extend(self.children.get(&key).into_iter().flatten().cloned());
        }
        children.sort();
        children.dedup();
        (children.len() <= MAX_EDGES_PER_MODULE).then_some(children)
    }

    fn parent_nodes(&self, file: &str) -> Option<Vec<String>> {
        if self.parent_overflow.contains(file) {
            None
        } else {
            self.parents.get(file).cloned()
        }
    }

    fn parent_nodes_or_inline(&self, file: &str) -> Option<Vec<String>> {
        if self.parent_overflow.contains(file) {
            None
        } else {
            Some(
                self.parents
                    .get(file)
                    .cloned()
                    .unwrap_or_else(|| vec![file.to_owned()]),
            )
        }
    }

    fn parents_of(&self, nodes: &[String]) -> Option<Vec<String>> {
        let mut parents = Vec::new();
        for node in nodes {
            parents.extend(self.parent_nodes(node)?);
        }
        parents.sort();
        parents.dedup();
        (!parents.is_empty() && parents.len() <= MAX_EDGES_PER_MODULE).then_some(parents)
    }

    fn root_nodes(&self, file: &str) -> Option<Vec<String>> {
        let mut current = vec![file.to_owned()];
        let mut seen = BTreeSet::new();
        loop {
            let mut parents = Vec::new();
            for node in &current {
                if !seen.insert(node.clone()) || self.parent_overflow.contains(node) {
                    return None;
                }
                parents.extend(self.parents.get(node).into_iter().flatten().cloned());
            }
            parents.sort();
            parents.dedup();
            if parents.is_empty() {
                return Some(current);
            }
            if parents.len() > MAX_EDGES_PER_MODULE {
                return None;
            }
            current = parents;
        }
    }

    fn insert_import(&mut self, file: &str, import: &MacroImportFact) {
        let key = (file.to_owned(), import.name.clone());
        insert_bounded(
            &mut self.imports,
            &mut self.import_overflow,
            key,
            import.clone(),
            MAX_IMPORTS_PER_NAME,
        );
    }

    fn insert_edge(&mut self, parent: &str, name: &str, child: &str) {
        insert_bounded(
            &mut self.children,
            &mut self.child_overflow,
            (parent.to_owned(), name.to_owned()),
            child.to_owned(),
            MAX_EDGES_PER_MODULE,
        );
        insert_bounded(
            &mut self.parents,
            &mut self.parent_overflow,
            child.to_owned(),
            parent.to_owned(),
            MAX_EDGES_PER_MODULE,
        );
    }
}

fn insert_bounded<K: Ord + Clone, V: Ord>(
    map: &mut BTreeMap<K, Vec<V>>,
    overflow: &mut BTreeSet<K>,
    key: K,
    value: V,
    limit: usize,
) {
    if overflow.contains(&key) {
        return;
    }
    let values = map.entry(key.clone()).or_default();
    match values.binary_search(&value) {
        Ok(_) => {}
        Err(index) if values.len() < limit => values.insert(index, value),
        Err(_) => {
            map.remove(&key);
            overflow.insert(key);
        }
    }
}
