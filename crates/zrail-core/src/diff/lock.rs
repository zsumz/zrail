//! Semantic comparison of generated exact state.

mod gates;
mod macros;

use std::collections::{BTreeMap, BTreeSet};

use crate::{LockFile, LockedDependency, LockedGeneratedSource, LockedPackage, LockedRatchet};

use super::{ArchitectureChange, ChangeKind};

pub(super) fn compare(
    before: Option<&LockFile>,
    after: Option<&LockFile>,
) -> Vec<ArchitectureChange> {
    match (before, after) {
        (Some(left), Some(right)) => compare_present(left, right),
        (None, Some(_)) => vec![ArchitectureChange::new(
            ChangeKind::Neutral,
            "lock",
            "zrail.lock",
            "resolved architecture state is now checked in",
        )],
        (Some(_), None) => vec![ArchitectureChange::new(
            ChangeKind::Grant,
            "lock",
            "zrail.lock",
            "resolved architecture state was removed",
        )],
        (None, None) => Vec::new(),
    }
}

fn compare_present(before: &LockFile, after: &LockFile) -> Vec<ArchitectureChange> {
    let mut changes = Vec::new();
    compare_packages(before, after, &mut changes);
    compare_edges(before, after, &mut changes);
    compare_generated(before, after, &mut changes);
    gates::compare(before, after, &mut changes);
    macros::compare(before, after, &mut changes);
    compare_ratchets(before, after, &mut changes);
    changes
}

fn compare_generated(before: &LockFile, after: &LockFile, changes: &mut Vec<ArchitectureChange>) {
    let old = generated_by_root(&before.generated);
    let new = generated_by_root(&after.generated);
    let roots = old
        .keys()
        .chain(new.keys())
        .copied()
        .collect::<BTreeSet<_>>();
    for root in roots {
        match (old.get(root), new.get(root)) {
            (None, Some(_)) => changes.push(ArchitectureChange::new(
                ChangeKind::Revoke,
                "rust.generated-provenance",
                root,
                "generated provenance became lock-protected",
            )),
            (Some(_), None) => changes.push(ArchitectureChange::new(
                ChangeKind::Grant,
                "rust.generated-provenance",
                root,
                "generated provenance is no longer lock-protected",
            )),
            (Some(left), Some(right)) if left != right => changes.push(
                ArchitectureChange::new(
                    ChangeKind::Unknown,
                    "rust.generated-provenance",
                    root,
                    "generated provenance manifest changed",
                )
                .values(*left, *right),
            ),
            _ => {}
        }
    }
}

fn compare_packages(before: &LockFile, after: &LockFile, changes: &mut Vec<ArchitectureChange>) {
    let old = package_names(&before.packages);
    let new = package_names(&after.packages);
    for package in new.difference(&old) {
        changes.push(ArchitectureChange::new(
            ChangeKind::Grant,
            "repository.package",
            package,
            "resolved workspace gained a package",
        ));
    }
    for package in old.difference(&new) {
        changes.push(ArchitectureChange::new(
            ChangeKind::Revoke,
            "repository.package",
            package,
            "resolved workspace lost a package",
        ));
    }
}

fn compare_edges(before: &LockFile, after: &LockFile, changes: &mut Vec<ArchitectureChange>) {
    let old = dependency_edges(&before.packages);
    let new = dependency_edges(&after.packages);
    for edge in new.difference(&old) {
        changes.push(ArchitectureChange::new(
            ChangeKind::Grant,
            "dependency.resolved-edge",
            edge_label(edge),
            "resolved dependency graph gained an edge",
        ));
    }
    for edge in old.difference(&new) {
        changes.push(ArchitectureChange::new(
            ChangeKind::Revoke,
            "dependency.resolved-edge",
            edge_label(edge),
            "resolved dependency graph lost an edge",
        ));
    }
}

fn compare_ratchets(before: &LockFile, after: &LockFile, changes: &mut Vec<ArchitectureChange>) {
    let old = ratchets_by_identity(&before.ratchets);
    let new = ratchets_by_identity(&after.ratchets);
    let identities = old
        .keys()
        .chain(new.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    for identity in identities {
        match (old.get(&identity), new.get(&identity)) {
            (Some(left), Some(right)) if left.value < right.value => changes.push(
                ArchitectureChange::new(
                    ChangeKind::Debt,
                    "ratchet",
                    &identity,
                    "ratchet ceiling increased",
                )
                .values(left.value.to_string(), right.value.to_string()),
            ),
            (Some(left), Some(right)) if left.value > right.value => changes.push(
                ArchitectureChange::new(
                    ChangeKind::Cleanup,
                    "ratchet",
                    &identity,
                    "ratchet tightened with the repository",
                )
                .values(left.value.to_string(), right.value.to_string()),
            ),
            (None, Some(right)) => changes.push(
                ArchitectureChange::new(
                    ChangeKind::Debt,
                    "ratchet",
                    &identity,
                    "reviewed architecture debt was recorded",
                )
                .values("<none>", right.value.to_string()),
            ),
            (Some(left), None) => changes.push(
                ArchitectureChange::new(
                    ChangeKind::Cleanup,
                    "ratchet",
                    &identity,
                    "reviewed architecture debt was removed",
                )
                .values(left.value.to_string(), "<none>"),
            ),
            _ => {}
        }
    }
}

fn package_names(packages: &[LockedPackage]) -> BTreeSet<String> {
    packages
        .iter()
        .map(|package| package.name.clone())
        .collect()
}

fn generated_by_root(generated: &[LockedGeneratedSource]) -> BTreeMap<&str, &str> {
    generated
        .iter()
        .map(|generated| (generated.root.as_str(), generated.manifest_sha256.as_str()))
        .collect()
}

fn dependency_edges(packages: &[LockedPackage]) -> BTreeSet<(String, LockedDependency)> {
    packages
        .iter()
        .flat_map(|package| {
            package
                .dependencies
                .iter()
                .cloned()
                .map(|dependency| (package.name.clone(), dependency))
        })
        .collect()
}

fn edge_label((package, dependency): &(String, LockedDependency)) -> String {
    format!("{package}->{}", dependency.label())
}

fn ratchets_by_identity(ratchets: &[LockedRatchet]) -> BTreeMap<String, &LockedRatchet> {
    ratchets
        .iter()
        .map(|ratchet| (format!("{}:{}", ratchet.rule, ratchet.target), ratchet))
        .collect()
}

#[cfg(test)]
#[path = "lock_test.rs"]
mod lock_test;
