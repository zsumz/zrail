//! Compilation paths turn file-local operation subjects into canonical Rust identities.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use zrail_core::AnalysisQuality;

use super::{CompilationModuleEdge, CompilationRoot, SourceIndex};

const MAX_MODULE_DEPTH: usize = 128;

pub(super) fn apply(
    index: &mut SourceIndex,
    roots: &[CompilationRoot],
    edges: &[CompilationModuleEdge],
) {
    let identities = module_identities(roots, edges);
    for file in &mut index.files {
        let modules = identities.get(&file.relative);
        for operation in &mut file.operations {
            if !operation.file_local {
                continue;
            }
            let candidates = modules
                .into_iter()
                .flatten()
                .map(|module| join_identity(module, &operation.identity.name))
                .collect::<Vec<_>>();
            match candidates.as_slice() {
                [identity] => operation.identity.name.clone_from(identity),
                [] => operation.identity.quality = AnalysisQuality::Unresolved,
                _ => {
                    operation.identity.canonical = candidates;
                    operation.identity.quality = AnalysisQuality::Conservative;
                }
            }
            operation.file_local = false;
        }
    }
}

fn module_identities(
    roots: &[CompilationRoot],
    edges: &[CompilationModuleEdge],
) -> BTreeMap<String, BTreeSet<String>> {
    let mut identities = BTreeMap::<String, BTreeSet<String>>::new();
    let mut queue = VecDeque::new();
    let mut visited = BTreeSet::new();
    for root in roots {
        queue.push_back((
            root.file.clone(),
            root.domain.clone(),
            "crate".to_owned(),
            0,
        ));
    }
    while let Some((file, domain, identity, depth)) = queue.pop_front() {
        if !visited.insert((file.clone(), domain.clone(), identity.clone())) {
            continue;
        }
        identities
            .entry(file.clone())
            .or_default()
            .insert(identity.clone());
        if depth == MAX_MODULE_DEPTH {
            continue;
        }
        for edge in edges.iter().filter(|edge| {
            edge.parent == file && edge.domain == domain && edge.parent_scope.is_empty()
        }) {
            queue.push_back((
                edge.child.clone(),
                edge.domain.clone(),
                format!("{identity}::{}", edge.module_name),
                depth + 1,
            ));
        }
    }
    identities
}

fn join_identity(module: &str, local: &str) -> String {
    if local.is_empty() {
        module.to_owned()
    } else {
        format!("{module}::{local}")
    }
}
