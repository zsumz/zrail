//! Package and dependency-edge lock drift remains exact and deterministic.

use std::collections::BTreeSet;

use zrail_core::{Finding, FindingSink, LockFile, LockedDependency, LockedPackage};

pub(super) fn compare(current: &LockFile, candidate: &LockFile, findings: &mut FindingSink) {
    compare_packages(current, candidate, findings);
    compare_edges(current, candidate, findings);
}

fn compare_packages(current: &LockFile, candidate: &LockFile, findings: &mut FindingSink) {
    let old = package_names(&current.packages);
    let new = package_names(&candidate.packages);
    for package in new.difference(&old) {
        findings.push(Finding::error(
            "LOCK-003",
            "lock.package",
            "lock",
            format!("workspace package {package:?} is not reviewed in zrail.lock"),
        ));
    }
    for package in old.difference(&new) {
        findings.push(Finding::error(
            "LOCK-004",
            "lock.package",
            "lock",
            format!("zrail.lock retains stale workspace package {package:?}"),
        ));
    }
}

fn compare_edges(current: &LockFile, candidate: &LockFile, findings: &mut FindingSink) {
    let old = package_edges(&current.packages);
    let new = package_edges(&candidate.packages);
    for edge in new.difference(&old) {
        findings.push(Finding::error(
            "LOCK-005",
            "lock.dependency-edge",
            "lock",
            format!(
                "resolved dependency edge {} is not reviewed in zrail.lock",
                edge_label(edge)
            ),
        ));
    }
    for edge in old.difference(&new) {
        findings.push(Finding::error(
            "LOCK-006",
            "lock.dependency-edge",
            "lock",
            format!(
                "zrail.lock retains stale dependency edge {}",
                edge_label(edge)
            ),
        ));
    }
}

fn package_names(packages: &[LockedPackage]) -> BTreeSet<String> {
    packages
        .iter()
        .map(|package| package.name.clone())
        .collect()
}

fn package_edges(packages: &[LockedPackage]) -> BTreeSet<(String, LockedDependency)> {
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
