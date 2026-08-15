//! Public semantic comparison entry point.

use crate::{Contract, LockFile};

use super::{ArchitectureChange, ChangeKind, DiffReport, contract, lock};

pub fn compare_architecture(
    before: &Contract,
    before_lock: Option<&LockFile>,
    after: &Contract,
    after_lock: Option<&LockFile>,
) -> DiffReport {
    let mut changes = contract::compare(before, after);
    changes.extend(lock::compare(before_lock, after_lock));
    DiffReport::new(changes)
}

pub fn compare_architecture_checked(
    before: &Contract,
    before_sha256: &str,
    before_lock: Option<&LockFile>,
    after: &Contract,
    after_sha256: &str,
    after_lock: Option<&LockFile>,
) -> DiffReport {
    let mut changes = contract::compare(before, after);
    let before_authority = lock_authority("before", before_sha256, before_lock);
    let after_authority = lock_authority("after", after_sha256, after_lock);
    changes.extend(before_authority);
    changes.extend(after_authority);
    if changes.iter().all(|change| change.rail != "lock.authority") {
        changes.extend(lock::compare(before_lock, after_lock));
    }
    DiffReport::new(changes)
}

fn lock_authority(
    side: &str,
    contract_sha256: &str,
    lock: Option<&LockFile>,
) -> Vec<ArchitectureChange> {
    let Some(lock) = lock else {
        return vec![ArchitectureChange::new(
            ChangeKind::Unknown,
            "lock.authority",
            format!("{side}:missing"),
            "architecture comparison has no lock authority for this repository state",
        )];
    };
    let mut changes = Vec::new();
    if lock.engine != env!("CARGO_PKG_VERSION") {
        changes.push(
            ArchitectureChange::new(
                ChangeKind::Unknown,
                "lock.authority",
                format!("{side}:engine"),
                "lock engine is incompatible with this zrail engine",
            )
            .values(&lock.engine, env!("CARGO_PKG_VERSION")),
        );
    }
    if lock.contract_sha256 != contract_sha256 {
        changes.push(
            ArchitectureChange::new(
                ChangeKind::Unknown,
                "lock.authority",
                format!("{side}:contract"),
                "lock was produced from different contract bytes",
            )
            .values(&lock.contract_sha256, contract_sha256),
        );
    }
    changes
}

#[cfg(test)]
#[path = "compare_test.rs"]
mod compare_test;
