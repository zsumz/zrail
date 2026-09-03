//! Public semantic comparison entry point.

use crate::{Contract, LOCK_SCHEMA, LOCK_SEMANTICS, LockFile};

use super::{ArchitectureChange, ChangeKind, DiffReport, contract, lock};

/// Compares effective contract policy and optional resolved lock state.
///
/// This compatibility entry point does not validate whether either lock belongs
/// to its contract. Use [`compare_architecture_checked`] whenever contract
/// digests are available. Returned changes and summary counts are deterministic.
pub fn compare_architecture(
    before: &Contract,
    before_lock: Option<&LockFile>,
    after: &Contract,
    after_lock: Option<&LockFile>,
) -> DiffReport {
    let mut changes = contract::compare(before, after);
    changes.extend(lock::compare(before, before_lock, after, after_lock));
    DiffReport::new(changes)
}

/// Compares contract policy and lock state with fail-closed authority checks.
///
/// Each supplied lock must use the supported schema and current semantics and
/// its `contract_sha256` must equal the corresponding digest argument. A missing,
/// stale, or incompatible lock contributes an [`ChangeKind::Unknown`] change on
/// `lock.authority`; resolved lock contents are compared only when both sides
/// have valid authority. Contract changes are always compared. The function does
/// no I/O and returns deterministically ordered changes.
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
        changes.extend(lock::compare(before, before_lock, after, after_lock));
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
    if !lock.has_supported_schema() {
        changes.push(
            ArchitectureChange::new(
                ChangeKind::Unknown,
                "lock.authority",
                format!("{side}:schema"),
                "lock schema is unsupported by this zrail engine",
            )
            .values(lock.schema.to_string(), LOCK_SCHEMA.to_string()),
        );
    }
    if !lock.has_current_semantics() {
        changes.push(
            ArchitectureChange::new(
                ChangeKind::Unknown,
                "lock.authority",
                format!("{side}:semantics"),
                "lock semantics are incompatible with this zrail engine",
            )
            .values(lock.semantics.to_string(), LOCK_SEMANTICS.to_string()),
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
