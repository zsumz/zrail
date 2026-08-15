//! Cycle detection emits one deterministic witness for each cyclic component.

use std::collections::{BTreeMap, BTreeSet};

use super::find_cycles;

#[test]
fn dense_component_produces_one_cycle_witness() {
    let names = ["a", "b", "c", "d", "e", "f", "g", "h"];
    let graph = names
        .iter()
        .map(|name| {
            let edges = names
                .iter()
                .copied()
                .filter(|candidate| candidate != name)
                .collect::<BTreeSet<_>>();
            (*name, edges)
        })
        .collect::<BTreeMap<_, _>>();

    let cycles = find_cycles(&graph);

    assert_eq!(cycles, [vec!["a", "b", "a"]]);
}

#[test]
fn separate_components_and_self_edges_have_stable_witnesses() {
    let graph = BTreeMap::from([
        ("a", BTreeSet::from(["b"])),
        ("b", BTreeSet::from(["a"])),
        ("c", BTreeSet::from(["d"])),
        ("d", BTreeSet::from(["c"])),
        ("e", BTreeSet::from(["e"])),
        ("f", BTreeSet::new()),
    ]);

    assert_eq!(
        find_cycles(&graph),
        [vec!["a", "b", "a"], vec!["c", "d", "c"], vec!["e", "e"]]
    );
}
