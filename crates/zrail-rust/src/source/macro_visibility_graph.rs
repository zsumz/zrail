//! Exact external-module edges expose only imports reachable through Rust module paths.

use std::collections::{BTreeMap, BTreeSet};

use super::{
    MacroImportFact, ModuleDeclaration, ModuleTarget, SourceIndex, SubmoduleBase, module_target,
};

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
    pub(super) fn collect(index: &SourceIndex) -> Self {
        let mut visibility = Self::default();
        let files = index
            .files
            .iter()
            .map(|file| file.relative.clone())
            .collect::<BTreeSet<_>>();
        for file in &index.files {
            for import in &file.macro_imports {
                visibility.insert_import(&file.relative, import);
            }
            for declaration in &file.modules {
                let targets = module_targets(&file.relative, declaration, &files);
                if targets.len() == 1 {
                    visibility.insert_edge(&file.relative, &declaration.name, &targets[0]);
                } else if targets.len() > 1 {
                    visibility
                        .child_overflow
                        .insert((file.relative.clone(), declaration.name.clone()));
                }
            }
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
            let mut next = Vec::new();
            for node in &nodes {
                let key = (node.clone(), (*module).to_owned());
                if self.child_overflow.contains(&key) {
                    return VisibilityLookup::Unknown;
                }
                next.extend(self.children.get(&key).into_iter().flatten().cloned());
            }
            next.sort();
            next.dedup();
            if next.is_empty() || next.len() > MAX_EDGES_PER_MODULE {
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
            Some("super") => self.parent_nodes(file),
            Some("crate") => self.root_nodes(file),
            _ => None,
        }
    }

    fn parent_nodes(&self, file: &str) -> Option<Vec<String>> {
        if self.parent_overflow.contains(file) {
            None
        } else {
            self.parents.get(file).cloned()
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

fn module_targets(
    source: &str,
    declaration: &ModuleDeclaration,
    files: &BTreeSet<String>,
) -> Vec<String> {
    let mut targets = BTreeSet::new();
    for base in [
        SubmoduleBase::SourceParent,
        SubmoduleBase::FileStemDirectory,
    ] {
        match module_target(source, base, declaration) {
            Ok(ModuleTarget::Exact(path)) if files.contains(&path) => {
                targets.insert(path);
            }
            Ok(ModuleTarget::Search { direct, nested }) => {
                targets.extend(
                    [direct, nested]
                        .into_iter()
                        .filter(|path| files.contains(path)),
                );
            }
            Ok(ModuleTarget::Exact(_)) | Err(_) => {}
        }
    }
    targets.into_iter().collect()
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
