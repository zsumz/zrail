//! Cargo feature-world changes have explicit analysis directionality.

use std::collections::{BTreeMap, BTreeSet};

use crate::{CargoFeaturePackageContract, CargoFeatureWorldContract, Contract};

use super::super::{ArchitectureChange, ChangeKind};

pub(super) fn compare(before: &Contract, after: &Contract, changes: &mut Vec<ArchitectureChange>) {
    let left = worlds(&before.source.rust.feature_worlds);
    let right = worlds(&after.source.rust.feature_worlds);
    if left.is_empty() != right.is_empty() {
        changes.push(ArchitectureChange::new(
            ChangeKind::Unknown,
            "rust.feature-world.mode",
            "source.rust.feature_worlds",
            "Cargo feature analysis changed between legacy conditional and exact worlds",
        ));
        return;
    }
    for name in left
        .keys()
        .chain(right.keys())
        .copied()
        .collect::<BTreeSet<_>>()
    {
        match (left.get(name), right.get(name)) {
            (None, Some(_)) => changes.push(ArchitectureChange::new(
                ChangeKind::Revoke,
                "rust.feature-world",
                name,
                "an exact Cargo feature compilation world is now analyzed",
            )),
            (Some(_), None) => changes.push(ArchitectureChange::new(
                ChangeKind::Grant,
                "rust.feature-world",
                name,
                "an exact Cargo feature compilation world is no longer analyzed",
            )),
            (Some(before), Some(after)) => compare_world(name, before, after, changes),
            (None, None) => {}
        }
    }
}

fn compare_world(
    world: &str,
    before: &CargoFeatureWorldContract,
    after: &CargoFeatureWorldContract,
    changes: &mut Vec<ArchitectureChange>,
) {
    let left = packages(before);
    let right = packages(after);
    for package in left
        .keys()
        .chain(right.keys())
        .copied()
        .collect::<BTreeSet<_>>()
    {
        let subject = format!("{world}:{package}");
        match (left.get(package), right.get(package)) {
            (None, Some(_)) | (Some(_), None) => changes.push(ArchitectureChange::new(
                ChangeKind::Unknown,
                "rust.feature-world.package",
                subject,
                "the complete workspace package map changed",
            )),
            (Some(before), Some(after)) => {
                compare_package(world, package, before, after, changes);
            }
            (None, None) => {}
        }
    }
}

fn compare_package(
    world: &str,
    package: &str,
    before: &CargoFeaturePackageContract,
    after: &CargoFeaturePackageContract,
    changes: &mut Vec<ArchitectureChange>,
) {
    let subject = format!("{world}:{package}");
    if before.default_features != after.default_features {
        changes.push(ArchitectureChange::new(
            ChangeKind::Unknown,
            "rust.feature-world.default-features",
            &subject,
            "default-feature analysis changed",
        ));
    }
    let left = before.features.iter().cloned().collect::<BTreeSet<_>>();
    let right = after.features.iter().cloned().collect::<BTreeSet<_>>();
    for feature in right.difference(&left) {
        changes.push(ArchitectureChange::new(
            ChangeKind::Unknown,
            "rust.feature-world.feature",
            format!("{subject}:{feature}"),
            "the feature world now analyzes this selected feature",
        ));
    }
    for feature in left.difference(&right) {
        changes.push(ArchitectureChange::new(
            ChangeKind::Unknown,
            "rust.feature-world.feature",
            format!("{subject}:{feature}"),
            "the feature world no longer analyzes this selected feature",
        ));
    }
}

fn worlds(values: &[CargoFeatureWorldContract]) -> BTreeMap<&str, &CargoFeatureWorldContract> {
    values
        .iter()
        .map(|world| (world.name.as_str(), world))
        .collect()
}

fn packages(world: &CargoFeatureWorldContract) -> BTreeMap<&str, &CargoFeaturePackageContract> {
    world
        .packages
        .iter()
        .map(|package| (package.package.as_str(), package))
        .collect()
}

#[cfg(test)]
#[path = "feature_worlds_test.rs"]
mod feature_worlds_test;
