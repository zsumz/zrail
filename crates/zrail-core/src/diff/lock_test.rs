//! Content-addressed gate diff classification.

use crate::{
    ChangeKind, LockFile, LockedExecutionReceipt, LockedGate, LockedGateInput, LockedRatchet,
};

use super::compare;
use crate::diff::compare_fixture_test::contract_with_hard_limit;

#[test]
fn changing_reviewed_gate_bytes_requires_human_review() {
    let before = lock("1");
    let after = lock("2");

    let changes = compare_locks(Some(&before), Some(&after));

    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].kind, ChangeKind::Unknown);
    assert_eq!(changes[0].rail, "qualification.gate-lock");
}

#[test]
fn removing_gate_attestation_is_a_grant() {
    let before = lock("1");
    let after = LockFile::new("0".repeat(64));

    let changes = compare_locks(Some(&before), Some(&after));

    assert!(
        changes
            .iter()
            .any(|change| change.kind == ChangeKind::Grant)
    );
}

#[test]
fn execution_receipt_lock_changes_preserve_permission_direction() {
    let mut before = LockFile::new("0".repeat(64));
    before.execution_receipts.push(receipt("1"));
    let mut changed = LockFile::new("0".repeat(64));
    changed.execution_receipts.push(receipt("2"));

    let reviewed = compare_locks(Some(&before), Some(&changed));
    assert_eq!(reviewed.len(), 1);
    assert_eq!(reviewed[0].kind, ChangeKind::Unknown);
    assert_eq!(reviewed[0].rail, "rust.test-mirror-receipt-lock");

    let empty = LockFile::new("0".repeat(64));
    assert_eq!(
        compare_locks(Some(&empty), Some(&before))[0].kind,
        ChangeKind::Revoke
    );
    assert_eq!(
        compare_locks(Some(&before), Some(&empty))[0].kind,
        ChangeKind::Grant
    );
}

#[test]
fn gate_input_changes_preserve_their_permission_direction() {
    let before = lock_with_input("1");
    let after = lock_with_input("2");
    let changed = compare_locks(Some(&before), Some(&after));
    assert_eq!(changed.len(), 1);
    assert_eq!(changed[0].kind, ChangeKind::Unknown);
    assert_eq!(changed[0].rail, "qualification.gate-input-lock");

    let without = lock("1");
    let added = compare_locks(Some(&without), Some(&before));
    assert_eq!(added[0].kind, ChangeKind::Revoke);
    let removed = compare_locks(Some(&before), Some(&without));
    assert_eq!(removed[0].kind, ChangeKind::Grant);
}

#[test]
fn ratchet_selector_is_normalized_and_part_of_identity() {
    let mut raw = LockFile::new("0".repeat(64));
    raw.ratchets.push(ratchet("r#unwrap"));
    let mut normalized = LockFile::new("0".repeat(64));
    normalized.ratchets.push(ratchet("unwrap"));
    assert!(compare_locks(Some(&raw), Some(&normalized)).is_empty());

    let mut changed = LockFile::new("0".repeat(64));
    changed.ratchets.push(ratchet("expect"));
    let changes = compare_locks(Some(&normalized), Some(&changed));
    assert_eq!(changes.len(), 2);
    assert!(
        changes
            .iter()
            .any(|change| change.subject.contains("[unwrap]"))
    );
    assert!(
        changes
            .iter()
            .any(|change| change.subject.contains("[expect]"))
    );
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

fn compare_locks(
    before: Option<&LockFile>,
    after: Option<&LockFile>,
) -> Vec<super::ArchitectureChange> {
    let contract = contract_with_hard_limit(300);
    compare(&contract, before, &contract, after)
}

fn lock_with_input(digit: &str) -> LockFile {
    let mut lock = lock("1");
    lock.gates[0].inputs.push(LockedGateInput {
        path: "scripts/helper".into(),
        sha256: digit.repeat(64),
    });
    lock
}

fn ratchet(selector: &str) -> LockedRatchet {
    LockedRatchet {
        rule: "rust.hygiene.denied-method".into(),
        selector: Some(selector.into()),
        target: "src/lib.rs".into(),
        value: 2,
    }
}

fn receipt(digit: &str) -> LockedExecutionReceipt {
    LockedExecutionReceipt {
        production: "src/state.rs".into(),
        test: "tests/state_test.rs".into(),
        name: "state_transitions".into(),
        receipt: "evidence/state.json".into(),
        sha256: digit.repeat(64),
        input_sha256: "3".repeat(64),
        producer: "runner 1.2.3".into(),
    }
}
