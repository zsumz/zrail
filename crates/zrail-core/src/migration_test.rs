//! Adjacent-epoch migration classifies every old and new authority subject.

use crate::{
    LOCK_SCHEMA, LOCK_SEMANTICS, LockFile, LockMigrationClassification, LockedExecutionReceipt,
    LockedGeneratedSource, compare_lock_epochs,
};

#[test]
fn adjacent_epoch_migration_is_scoped_per_authority_subject() {
    let digest = "0".repeat(64);
    let mut old = LockFile::new(&digest);
    old.schema = LOCK_SCHEMA - 1;
    old.semantics = LOCK_SEMANTICS - 1;
    old.analysis = None;
    old.generated.push(LockedGeneratedSource {
        root: "generated".into(),
        manifest_sha256: "1".repeat(64),
    });
    let mut new = LockFile::new(digest);
    new.generated.push(LockedGeneratedSource {
        root: "generated".into(),
        manifest_sha256: "2".repeat(64),
    });
    new.execution_receipts.push(LockedExecutionReceipt {
        production: "src/state.rs".into(),
        test: "tests/state_test.rs".into(),
        name: "state_transitions".into(),
        receipt: "evidence/state-transitions.json".into(),
        sha256: "3".repeat(64),
        input_sha256: "4".repeat(64),
        producer: "test-runner 1.2.3".into(),
    });

    let report = compare_lock_epochs(&old, &new).expect("compare adjacent epoch");

    assert!(report.entries.iter().any(|entry| {
        entry.rail == "rust.generated-provenance"
            && entry.classification == LockMigrationClassification::ChangedInterpretation
    }));
    assert!(report.entries.iter().any(|entry| {
        entry.rail == "analysis.inventory"
            && entry.classification == LockMigrationClassification::NewlyObservable
    }));
    assert!(report.entries.iter().any(|entry| {
        entry.rail == "rust.test-mirror-receipt-lock"
            && entry.classification == LockMigrationClassification::NewlyObservable
    }));
    assert!(report.summary.preserved > 0);
    assert_eq!(report.summary.changed_interpretation, 1);
}

#[test]
fn migration_rejects_nonadjacent_or_different_contract_authority() {
    let mut old = LockFile::new("0".repeat(64));
    old.schema = LOCK_SCHEMA - 1;
    old.semantics = LOCK_SEMANTICS - 2;
    let new = LockFile::new("0".repeat(64));
    assert!(compare_lock_epochs(&old, &new).is_err());

    old.semantics = LOCK_SEMANTICS - 1;
    let changed = LockFile::new("1".repeat(64));
    assert!(compare_lock_epochs(&old, &changed).is_err());
}
