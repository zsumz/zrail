//! Deterministic workspace dependency-cycle discovery.

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::{Finding, FindingSink};

use crate::cargo::CargoWorkspace;

pub(super) fn check_cycles(workspace: &CargoWorkspace, findings: &mut FindingSink) {
    let package_names = workspace
        .packages
        .iter()
        .map(|package| package.name.as_str())
        .collect::<BTreeSet<_>>();
    let graph = workspace
        .packages
        .iter()
        .map(|package| {
            let dependencies = package
                .dependencies
                .iter()
                .filter(|dependency| package_names.contains(dependency.name.as_str()))
                .map(|dependency| dependency.name.as_str())
                .collect::<BTreeSet<_>>();
            (package.name.as_str(), dependencies)
        })
        .collect::<BTreeMap<_, _>>();
    for cycle in find_cycles(&graph) {
        findings.push(Finding::error(
            "DEP-009",
            "dependency.cycles",
            "dependency",
            format!("workspace dependency cycle: {}", cycle.join(" -> ")),
        ));
    }
}

fn find_cycles<'a>(graph: &BTreeMap<&'a str, BTreeSet<&'a str>>) -> Vec<Vec<&'a str>> {
    let mut cycles = BTreeSet::new();
    for start in graph.keys().copied() {
        walk_cycle(
            start,
            start,
            graph,
            &mut Vec::new(),
            &mut BTreeSet::new(),
            &mut cycles,
        );
    }
    cycles.into_iter().collect()
}

fn walk_cycle<'a>(
    start: &'a str,
    current: &'a str,
    graph: &BTreeMap<&'a str, BTreeSet<&'a str>>,
    path: &mut Vec<&'a str>,
    active: &mut BTreeSet<&'a str>,
    cycles: &mut BTreeSet<Vec<&'a str>>,
) {
    path.push(current);
    active.insert(current);
    if let Some(edges) = graph.get(current) {
        for next in edges {
            if *next == start {
                let mut cycle = path.clone();
                cycle.push(start);
                cycles.insert(canonical_cycle(cycle));
            } else if !active.contains(next) {
                walk_cycle(start, next, graph, path, active, cycles);
            }
        }
    }
    active.remove(current);
    path.pop();
}

fn canonical_cycle(mut cycle: Vec<&str>) -> Vec<&str> {
    cycle.pop();
    let index = cycle
        .iter()
        .enumerate()
        .min_by_key(|(_, value)| *value)
        .map_or(0, |(index, _)| index);
    cycle.rotate_left(index);
    cycle.push(cycle[0]);
    cycle
}
