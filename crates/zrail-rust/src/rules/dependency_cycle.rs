//! Deterministic workspace dependency-cycle discovery.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use zrail_core::{Finding, FindingSink};

use crate::cargo::CargoWorkspace;

pub(super) fn check_cycles(workspace: &CargoWorkspace, findings: &mut FindingSink) {
    let graph = workspace
        .packages
        .iter()
        .map(|package| {
            let dependencies = package
                .dependencies
                .iter()
                .filter_map(|dependency| dependency.internal_package())
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
    strongly_connected_components(graph)
        .into_iter()
        .filter(|component| {
            component.len() > 1
                || graph
                    .get(component[0])
                    .is_some_and(|edges| edges.contains(component[0]))
        })
        .filter_map(|component| witness_cycle(graph, &component))
        .collect()
}

fn strongly_connected_components<'a>(
    graph: &BTreeMap<&'a str, BTreeSet<&'a str>>,
) -> Vec<Vec<&'a str>> {
    let mut visited = BTreeSet::new();
    let mut finished = Vec::new();
    for start in graph.keys().copied() {
        if visited.contains(start) {
            continue;
        }
        let mut stack = vec![(start, false)];
        while let Some((node, expanded)) = stack.pop() {
            if expanded {
                finished.push(node);
            } else if visited.insert(node) {
                stack.push((node, true));
                if let Some(edges) = graph.get(node) {
                    for next in edges.iter().rev() {
                        if !visited.contains(next) {
                            stack.push((next, false));
                        }
                    }
                }
            }
        }
    }
    let reverse = reverse_graph(graph);
    visited.clear();
    let mut components = Vec::new();
    for start in finished.into_iter().rev() {
        if !visited.insert(start) {
            continue;
        }
        let mut component = Vec::new();
        let mut stack = vec![start];
        while let Some(node) = stack.pop() {
            component.push(node);
            for next in reverse.get(node).into_iter().flatten().rev() {
                if visited.insert(next) {
                    stack.push(next);
                }
            }
        }
        component.sort_unstable();
        components.push(component);
    }
    components.sort_by_key(|component| component[0]);
    components
}

fn reverse_graph<'a>(
    graph: &BTreeMap<&'a str, BTreeSet<&'a str>>,
) -> BTreeMap<&'a str, BTreeSet<&'a str>> {
    let mut reverse = graph
        .keys()
        .map(|node| (*node, BTreeSet::new()))
        .collect::<BTreeMap<_, _>>();
    for (from, edges) in graph {
        for to in edges {
            reverse.entry(*to).or_default().insert(*from);
        }
    }
    reverse
}

fn witness_cycle<'a>(
    graph: &BTreeMap<&'a str, BTreeSet<&'a str>>,
    component: &[&'a str],
) -> Option<Vec<&'a str>> {
    let start = component[0];
    if component.len() == 1 {
        return Some(vec![start, start]);
    }
    let members = component.iter().copied().collect::<BTreeSet<_>>();
    for next in graph.get(start)?.iter().filter(|next| **next != start) {
        if members.contains(next)
            && let Some(path) = path_within(graph, next, start, &members)
        {
            let mut cycle = vec![start];
            cycle.extend(path);
            return Some(cycle);
        }
    }
    None
}

fn path_within<'a>(
    graph: &BTreeMap<&'a str, BTreeSet<&'a str>>,
    from: &'a str,
    goal: &'a str,
    members: &BTreeSet<&'a str>,
) -> Option<Vec<&'a str>> {
    let mut queue = VecDeque::from([from]);
    let mut parent: BTreeMap<&'a str, Option<&'a str>> = BTreeMap::from([(from, None)]);
    while let Some(node) = queue.pop_front() {
        if node == goal {
            let mut path = vec![goal];
            let mut cursor = goal;
            while let Some(Some(previous)) = parent.get(cursor) {
                path.push(*previous);
                cursor = *previous;
            }
            path.reverse();
            return Some(path);
        }
        for next in graph.get(node).into_iter().flatten() {
            if members.contains(next) && !parent.contains_key(next) {
                parent.insert(*next, Some(node));
                queue.push_back(*next);
            }
        }
    }
    None
}

#[cfg(test)]
#[path = "dependency_cycle_test.rs"]
mod dependency_cycle_test;
