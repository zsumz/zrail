//! Global dependency modes and per-layer policy changes.

use std::collections::{BTreeMap, BTreeSet};

use crate::Contract;

use super::{
    ArchitectureChange, ChangeKind,
    support::{
        compare_ordered_mode, compare_set_values, rank_cycles, rank_dependencies,
        rank_external_dependencies, rank_policy,
    },
};

pub(super) fn compare_dependency_modes(
    before: &Contract,
    after: &Contract,
    changes: &mut Vec<ArchitectureChange>,
) {
    compare_ordered_mode(
        "dependency.lock",
        "dependencies.mode",
        rank_dependencies(before.dependencies.mode),
        rank_dependencies(after.dependencies.mode),
        changes,
    );
    compare_ordered_mode(
        "dependency.assignment",
        "dependencies.unassigned_packages",
        rank_policy(before.dependencies.unassigned_packages),
        rank_policy(after.dependencies.unassigned_packages),
        changes,
    );
    compare_ordered_mode(
        "dependency.cycles",
        "dependencies.cycles",
        rank_cycles(before.dependencies.cycles),
        rank_cycles(after.dependencies.cycles),
        changes,
    );
}

pub(super) fn compare_crate_roots(
    before: &Contract,
    after: &Contract,
    changes: &mut Vec<ArchitectureChange>,
) {
    let roots = |contract: &Contract| {
        contract
            .dependencies
            .crate_roots
            .iter()
            .map(|attestation| {
                (
                    format!("{}@{}", attestation.package, attestation.source.identity()),
                    attestation.root.clone(),
                )
            })
            .collect::<BTreeMap<_, _>>()
    };
    let old = roots(before);
    let new = roots(after);
    for identity in old
        .keys()
        .chain(new.keys())
        .cloned()
        .collect::<BTreeSet<_>>()
    {
        match (old.get(&identity), new.get(&identity)) {
            (None, Some(root)) => changes.push(ArchitectureChange::new(
                ChangeKind::Grant,
                "dependency.crate-root",
                format!("{identity}:{root}"),
                "contract now trusts an external package crate-root identity",
            )),
            (Some(root), None) => changes.push(ArchitectureChange::new(
                ChangeKind::Revoke,
                "dependency.crate-root",
                format!("{identity}:{root}"),
                "contract no longer trusts an external package crate-root identity",
            )),
            (Some(left), Some(right)) if left != right => changes.push(
                ArchitectureChange::new(
                    ChangeKind::Unknown,
                    "dependency.crate-root",
                    identity,
                    "attested external crate-root identity changed",
                )
                .values(left, right),
            ),
            _ => {}
        }
    }
}

pub(super) fn compare_layer_profiles(
    before: &Contract,
    after: &Contract,
    changes: &mut Vec<ArchitectureChange>,
) {
    compare_set_values(
        "effect.layer-profile",
        &layer_profiles(before),
        &layer_profiles(after),
        ChangeKind::Revoke,
        ChangeKind::Grant,
        "applies an effect profile to a layer",
        changes,
    );
}

pub(super) fn compare_layer_external_modes(
    before: &Contract,
    after: &Contract,
    changes: &mut Vec<ArchitectureChange>,
) {
    let old = external_modes(before);
    let new = external_modes(after);
    for (&name, &old_mode) in &old {
        let Some(&new_mode) = new.get(name) else {
            continue;
        };
        compare_ordered_mode(
            "dependency.external",
            name,
            rank_external_dependencies(old_mode),
            rank_external_dependencies(new_mode),
            changes,
        );
    }
}

fn layer_profiles(contract: &Contract) -> BTreeSet<String> {
    contract
        .layers
        .iter()
        .flat_map(|layer| {
            layer
                .profiles
                .iter()
                .map(move |profile| format!("{}:{profile}", layer.name))
        })
        .collect()
}

fn external_modes(contract: &Contract) -> BTreeMap<&str, crate::ExternalDependencyMode> {
    contract
        .layers
        .iter()
        .map(|layer| (layer.name.as_str(), layer.dependencies.external))
        .collect()
}
