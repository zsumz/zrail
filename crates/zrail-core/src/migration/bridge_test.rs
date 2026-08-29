//! Bridge identities change when either revision or any changed file changes.

use crate::{LockFile, compare_lock_epochs_across_revisions};

use super::{
    LockMigrationBridgeReport, LockMigrationFileChange, LockMigrationFileState,
    LockMigrationRevision,
};

#[test]
fn bridge_digest_binds_revisions_and_repository_changes() {
    let mut old = LockFile::new("0".repeat(64));
    old.schema = 1;
    old.semantics = 1;
    old.analysis = None;
    let current = LockFile::new("1".repeat(64));
    let migration =
        compare_lock_epochs_across_revisions(&old, &current).expect("compare revisions");
    let report = LockMigrationBridgeReport {
        schema: 1,
        base: revision('a', '0', '2'),
        target: revision('b', '1', '3'),
        base_analysis_error: "source cannot be analyzed".into(),
        changes: vec![LockMigrationFileChange {
            path: "src/lib.rs".into(),
            before: Some(file('4')),
            after: Some(file('5')),
        }],
        migration,
    };

    let digest = report.sha256();
    let mut changed = report.clone();
    changed.changes[0].after = Some(file('6'));

    assert_ne!(digest, changed.sha256());
    assert_eq!(
        report.json().expect("render"),
        report.json().expect("repeat")
    );
}

fn revision(commit: char, contract: char, lock: char) -> LockMigrationRevision {
    LockMigrationRevision {
        commit: commit.to_string().repeat(40),
        contract_sha256: contract.to_string().repeat(64),
        lock_sha256: lock.to_string().repeat(64),
    }
}

fn file(digest: char) -> LockMigrationFileState {
    LockMigrationFileState {
        mode: "100644".into(),
        sha256: digest.to_string().repeat(64),
    }
}
