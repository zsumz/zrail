//! Exact dependency-path resolution is shared by enforcement and coverage reports.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use zrail_core::{DependencyEdgeKind, DependencyReachability, DependencyRule};

use crate::cargo::{DependencyKind, Package, ResolvedCargoGraph, ResolvedPackageIdentity};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct ResolvedDependencyPath {
    pub(crate) kind: DependencyKind,
    pub(crate) nodes: Vec<ResolvedPackageIdentity>,
}

pub(crate) fn resolve_denied_paths(
    package: &Package,
    rule: &DependencyRule,
    graph: &ResolvedCargoGraph,
) -> Result<Vec<ResolvedDependencyPath>, String> {
    let root = graph.workspace_package(&package.name).ok_or_else(|| {
        format!(
            "Cargo.lock contains no local node for workspace package {:?}",
            package.name
        )
    })?;
    let mut starts = BTreeSet::new();
    for dependency in package
        .dependencies
        .iter()
        .filter(|dependency| selected_kind(rule, dependency.kind))
    {
        starts.insert((
            dependency.kind,
            graph.manifest_dependency(package, dependency)?,
        ));
    }
    Ok(shortest_paths(root, starts, rule, graph))
}

fn shortest_paths(
    root: &ResolvedPackageIdentity,
    starts: BTreeSet<(DependencyKind, ResolvedPackageIdentity)>,
    rule: &DependencyRule,
    graph: &ResolvedCargoGraph,
) -> Vec<ResolvedDependencyPath> {
    let denied = rule
        .deny
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let mut queue = VecDeque::new();
    let mut visited = BTreeSet::from([(*root).clone()]);
    for (kind, target) in starts {
        if visited.insert(target.clone()) {
            queue.push_back((
                target.clone(),
                ResolvedDependencyPath {
                    kind,
                    nodes: vec![(*root).clone(), target],
                },
            ));
        }
    }
    let mut violations = BTreeMap::new();
    while let Some((current, path)) = queue.pop_front() {
        if denied.contains(current.name.as_str()) {
            violations.entry(current.clone()).or_insert(path.clone());
        }
        if rule.reachability == DependencyReachability::Direct {
            continue;
        }
        for dependency in graph.dependencies(&current) {
            if visited.insert(dependency.clone()) {
                let mut next = path.clone();
                next.nodes.push(dependency.clone());
                queue.push_back((dependency.clone(), next));
            }
        }
    }
    violations.into_values().collect()
}

pub(crate) fn selected_kind(rule: &DependencyRule, kind: DependencyKind) -> bool {
    rule.kinds.is_empty() || rule.kinds.contains(&edge_kind(kind))
}

pub(crate) const fn edge_kind(kind: DependencyKind) -> DependencyEdgeKind {
    match kind {
        DependencyKind::Normal => DependencyEdgeKind::Normal,
        DependencyKind::Development => DependencyEdgeKind::Development,
        DependencyKind::Build => DependencyEdgeKind::Build,
    }
}

pub(crate) const fn dependency_kind(kind: DependencyKind) -> &'static str {
    match kind {
        DependencyKind::Normal => "normal",
        DependencyKind::Development => "development",
        DependencyKind::Build => "build",
    }
}
