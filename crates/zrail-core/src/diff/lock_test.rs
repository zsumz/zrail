//! Content-addressed gate diff classification.

use crate::{ChangeKind, LockFile, LockedGate, LockedGateInput};

use super::compare;

#[test]
fn changing_reviewed_gate_bytes_requires_human_review() {
    let before = lock("1");
    let after = lock("2");

    let changes = compare(Some(&before), Some(&after));

    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].kind, ChangeKind::Unknown);
    assert_eq!(changes[0].rail, "qualification.gate-lock");
}

#[test]
fn removing_gate_attestation_is_a_grant() {
    let before = lock("1");
    let after = LockFile::new("0".repeat(64));

    let changes = compare(Some(&before), Some(&after));

    assert!(
        changes
            .iter()
            .any(|change| change.kind == ChangeKind::Grant)
    );
}

#[test]
fn gate_input_changes_preserve_their_permission_direction() {
    let before = lock_with_input("1");
    let after = lock_with_input("2");
    let changed = compare(Some(&before), Some(&after));
    assert_eq!(changed.len(), 1);
    assert_eq!(changed[0].kind, ChangeKind::Unknown);
    assert_eq!(changed[0].rail, "qualification.gate-input-lock");

    let without = lock("1");
    let added = compare(Some(&without), Some(&before));
    assert_eq!(added[0].kind, ChangeKind::Revoke);
    let removed = compare(Some(&before), Some(&without));
    assert_eq!(removed[0].kind, ChangeKind::Grant);
}

fn lock(digit: &str) -> LockFile {
    let mut lock = LockFile::new("0".repeat(64));
    lock.gates.push(LockedGate {
        name: "check".into(),
        path: "scripts/check".into(),
        sha256: digit.repeat(64),
        inputs: Vec::new(),
    });
    lock
}

fn lock_with_input(digit: &str) -> LockFile {
    let mut lock = lock("1");
    lock.gates[0].inputs.push(LockedGateInput {
        path: "scripts/helper".into(),
        sha256: digit.repeat(64),
    });
    lock
}
