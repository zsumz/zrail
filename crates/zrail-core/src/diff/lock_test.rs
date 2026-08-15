//! Content-addressed gate diff classification.

use crate::{ChangeKind, LockFile, LockedGate};

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

fn lock(digit: &str) -> LockFile {
    let mut lock = LockFile::new("0".repeat(64));
    lock.gates.push(LockedGate {
        name: "check".into(),
        path: "scripts/check".into(),
        sha256: digit.repeat(64),
    });
    lock
}
