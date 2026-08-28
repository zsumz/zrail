//! Exact external-module edges expose only imports reachable through Rust module paths.

use std::collections::{BTreeMap, BTreeSet};

use super::macro_visibility_reachability::{
    Edges, VisibilityKey, VisibilityNode, bounded_nodes, insert_reachable, intersect_edges,
};
use super::{MacroImportFact, Reachability, SourceSyntax};

#[derive(Default)]
pub(super) struct MacroVisibility {
    pub(super) imports: BTreeMap<(String, SourceSyntax, String), Vec<MacroImportFact>>,
    pub(super) import_overflow: BTreeSet<(String, SourceSyntax, String)>,
    pub(super) children: BTreeMap<(String, SourceSyntax, String), Edges>,
    pub(super) child_overflow: BTreeSet<(String, SourceSyntax, String)>,
    pub(super) parents: BTreeMap<VisibilityKey, Edges>,
    pub(super) parent_overflow: BTreeSet<VisibilityKey>,
}

pub(super) enum VisibilityLookup<'a> {
    Known(Vec<&'a MacroImportFact>),
    Unknown,
}

impl MacroVisibility {
    pub(super) fn imports_for<'a>(
        &'a self,
        file: &str,
        syntax: SourceSyntax,
        path: &str,
        reachability: Reachability,
    ) -> VisibilityLookup<'a> {
        let segments = path.split("::").collect::<Vec<_>>();
        let Some((name, prefix)) = segments.split_last() else {
            return VisibilityLookup::Unknown;
        };
        let root = prefix.first().copied();
        let Some(mut nodes) = self.start_nodes(file, syntax, root, reachability) else {
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
        for node in nodes
            .into_iter()
            .filter(|node| node.reachability.covers(reachability))
        {
            let key = (node.file, node.syntax, (*name).to_owned());
            if self.import_overflow.contains(&key) {
                return VisibilityLookup::Unknown;
            }
            imports.extend(self.imports.get(&key).into_iter().flatten());
        }
        VisibilityLookup::Known(imports)
    }

    fn start_nodes(
        &self,
        file: &str,
        syntax: SourceSyntax,
        root: Option<&str>,
        reachability: Reachability,
    ) -> Option<Vec<VisibilityNode>> {
        let node = VisibilityNode {
            file: file.to_owned(),
            syntax,
            reachability,
        };
        match root {
            Some("self") => Some(vec![node]),
            Some("super") => self.parent_nodes_or_inline(&node),
            Some("crate") => self.root_nodes(node),
            Some(module) => self.child_nodes(&[node], module),
            None => None,
        }
    }

    pub(super) fn repository_candidate(
        &self,
        file: &str,
        syntax: SourceSyntax,
        path: &str,
    ) -> bool {
        if super::macro_visibility::repository_path(path) {
            return true;
        }
        let Some(root) = path.split("::").next() else {
            return false;
        };
        let key = (file.to_owned(), syntax, root.to_owned());
        self.children.contains_key(&key) || self.child_overflow.contains(&key)
    }

    fn child_nodes(&self, nodes: &[VisibilityNode], module: &str) -> Option<Vec<VisibilityNode>> {
        let mut children = Edges::new();
        for node in nodes {
            let key = (node.file.clone(), node.syntax, module.to_owned());
            if self.child_overflow.contains(&key) {
                return None;
            }
            for (child, edge_reachability) in self.children.get(&key).into_iter().flatten() {
                insert_reachable(
                    &mut children,
                    child,
                    node.reachability.intersection(*edge_reachability),
                );
            }
        }
        bounded_nodes(children)
    }

    fn parent_nodes(&self, node: &VisibilityNode) -> Option<Vec<VisibilityNode>> {
        let key = (node.file.clone(), node.syntax);
        if self.parent_overflow.contains(&key) {
            return None;
        }
        let edges = self.parents.get(&key)?;
        bounded_nodes(intersect_edges(edges, node.reachability))
    }

    fn parent_nodes_or_inline(&self, node: &VisibilityNode) -> Option<Vec<VisibilityNode>> {
        let key = (node.file.clone(), node.syntax);
        if self.parent_overflow.contains(&key) {
            None
        } else {
            self.parents.get(&key).map_or_else(
                || Some(vec![node.clone()]),
                |edges| bounded_nodes(intersect_edges(edges, node.reachability)),
            )
        }
    }

    fn parents_of(&self, nodes: &[VisibilityNode]) -> Option<Vec<VisibilityNode>> {
        let mut parents = Edges::new();
        for node in nodes {
            for parent in self.parent_nodes(node)? {
                insert_reachable(
                    &mut parents,
                    &(parent.file.clone(), parent.syntax),
                    parent.reachability,
                );
            }
        }
        if parents.is_empty() {
            None
        } else {
            bounded_nodes(parents)
        }
    }

    fn root_nodes(&self, node: VisibilityNode) -> Option<Vec<VisibilityNode>> {
        let mut current = vec![node];
        let mut seen = BTreeSet::new();
        loop {
            let mut parents = Edges::new();
            let mut roots = Vec::new();
            for node in &current {
                let key = (node.file.clone(), node.syntax);
                if !seen.insert(node.clone()) || self.parent_overflow.contains(&key) {
                    return None;
                }
                let Some(edges) = self.parents.get(&key) else {
                    roots.push(node.clone());
                    continue;
                };
                for (parent, edge_reachability) in edges {
                    insert_reachable(
                        &mut parents,
                        parent,
                        node.reachability.intersection(*edge_reachability),
                    );
                }
            }
            if !roots.is_empty() {
                return parents.is_empty().then_some(roots);
            }
            if parents.is_empty() {
                return Some(Vec::new());
            }
            current = bounded_nodes(parents)?;
        }
    }
}
