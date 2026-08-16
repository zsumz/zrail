//! Exact lock drift comparison for dependencies, provenance, engine state, and ratchets.

mod gates;
mod generated;
mod macros;

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::{
    Contract, DependencyMode, Finding, FindingSink, LOCK_SCHEMA, LOCK_SEMANTICS, LockFile,
    LockedDependency, LockedPackage,
};

use super::model::RepositoryModel;

pub(super) fn check_lock(
    model: &RepositoryModel,
    current: Option<&LockFile>,
    candidate: &LockFile,
    findings: &mut FindingSink,
) {
    if !requires_lock(&model.bundle.contract) {
        return;
    }
    let Some(current) = current else {
        findings.push(
            Finding::error(
                "LOCK-001",
                "lock.required",
                "lock",
                "declared exact or ratcheted architecture state requires zrail.lock",
            )
            .with_help("run `zrail update` and review the resolved architecture state"),
        );
        return;
    };
    if !current.has_supported_schema() {
        findings.push(
            Finding::error(
                "LOCK-020",
                "lock.schema",
                "lock",
                format!(
                    "zrail.lock uses schema {}, latest supported schema is {}",
                    current.schema, LOCK_SCHEMA
                ),
            )
            .with_help("use a zrail engine that understands this lock schema"),
        );
    }
    if !current.has_current_semantics() {
        findings.push(
            Finding::error(
                "LOCK-008",
                "lock.semantics",
                "lock",
                format!(
                    "zrail.lock uses semantics {}, current engine uses semantics {}",
                    current.semantics, LOCK_SEMANTICS
                ),
            )
            .with_help("review the semantic migration with a compatible zrail engine"),
        );
    }
    if current.contract_sha256 != model.bundle.sha256 {
        findings.push(
            Finding::error(
                "LOCK-002",
                "lock.contract",
                "lock",
                "zrail.lock was produced from different contract bytes",
            )
            .with_help("run `zrail diff` before updating the lock"),
        );
    }
    compare_locks(current, candidate, findings);
}

fn compare_locks(current: &LockFile, candidate: &LockFile, findings: &mut FindingSink) {
    compare_packages(current, candidate, findings);
    compare_edges(current, candidate, findings);
    generated::compare(current, candidate, findings);
    gates::compare(current, candidate, findings);
    macros::compare(current, candidate, findings);
    compare_ratchets(current, candidate, findings);
}

pub(super) fn requires_lock(contract: &Contract) -> bool {
    contract.dependencies.mode == DependencyMode::Locked
        || !contract.source.rust.generated.is_empty()
        || !contract.gates.is_empty()
        || !contract.source.rust.macros.allow.is_empty()
        || !contract.ratchets.is_empty()
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

fn compare_ratchets(current: &LockFile, candidate: &LockFile, findings: &mut FindingSink) {
    let old = ratchet_values(current);
    let new = ratchet_values(candidate);
    for identity in new.keys().filter(|identity| !old.contains_key(*identity)) {
        findings.push(Finding::error(
            "LOCK-009",
            "lock.ratchet",
            "lock",
            format!("repository ratchet {identity:?} is not reviewed in zrail.lock"),
        ));
    }
    for identity in old.keys().filter(|identity| !new.contains_key(*identity)) {
        findings.push(Finding::error(
            "LOCK-007",
            "lock.ratchet",
            "lock",
            format!("zrail.lock retains stale ratchet {identity:?}"),
        ));
    }
    for (identity, old_value) in &old {
        let Some(new_value) = new.get(identity) else {
            continue;
        };
        if old_value != new_value {
            findings.push(Finding::error(
                "LOCK-010",
                "lock.ratchet",
                "lock",
                format!(
                    "ratchet {identity:?} records {old_value} but repository \
                     resolves to {new_value}"
                ),
            ));
        }
    }
}

fn package_names(packages: &[LockedPackage]) -> BTreeSet<String> {
    packages
        .iter()
        .map(|package| package.name.clone())
        .collect()
}

fn ratchet_values(lock: &LockFile) -> BTreeMap<String, usize> {
    lock.ratchets
        .iter()
        .map(|ratchet| {
            (
                format!("{}:{}", ratchet.rule, ratchet.target),
                ratchet.value,
            )
        })
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

#[cfg(test)]
#[path = "lock_compare_test.rs"]
mod lock_compare_test;
