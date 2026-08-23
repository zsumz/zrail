//! Collection of bounded imports and exact module edges for macro visibility.

use std::collections::{BTreeMap, BTreeSet};

use super::macro_visibility_graph::MacroVisibility;
use super::macro_visibility_reachability::insert_edge_bounded;
use super::{MacroImportFact, ResolvedModuleEdge, SourceIndex};

const MAX_IMPORTS_PER_NAME: usize = 4;

impl MacroVisibility {
    pub(super) fn collect(index: &SourceIndex, module_edges: &[ResolvedModuleEdge]) -> Self {
        let mut visibility = Self::default();
        for file in &index.files {
            for import in &file.macro_imports {
                visibility.insert_import(&file.relative, import);
            }
        }
        for edge in module_edges {
            visibility.insert_edge(
                &edge.parent,
                &edge.module_name,
                &edge.child,
                edge.reachability,
            );
        }
        visibility
    }

    fn insert_import(&mut self, file: &str, import: &MacroImportFact) {
        insert_bounded(
            &mut self.imports,
            &mut self.import_overflow,
            (file.to_owned(), import.name.clone()),
            import.clone(),
        );
    }

    fn insert_edge(
        &mut self,
        parent: &str,
        name: &str,
        child: &str,
        reachability: super::Reachability,
    ) {
        insert_edge_bounded(
            &mut self.children,
            &mut self.child_overflow,
            (parent.to_owned(), name.to_owned()),
            child,
            reachability,
        );
        insert_edge_bounded(
            &mut self.parents,
            &mut self.parent_overflow,
            child.to_owned(),
            parent,
            reachability,
        );
    }
}

fn insert_bounded<K: Ord + Clone, V: Ord>(
    map: &mut BTreeMap<K, Vec<V>>,
    overflow: &mut BTreeSet<K>,
    key: K,
    value: V,
) {
    if overflow.contains(&key) {
        return;
    }
    let values = map.entry(key.clone()).or_default();
    match values.binary_search(&value) {
        Ok(_) => {}
        Err(index) if values.len() < MAX_IMPORTS_PER_NAME => values.insert(index, value),
        Err(_) => {
            map.remove(&key);
            overflow.insert(key);
        }
    }
}
