//! Protected review rejects incompatible lock authority explicitly.

use zrail_core::LockFile;

use super::{
    git_base::git_available,
    review,
    review_fixture::{fixture, options, reset},
};

#[test]
fn changed_producer_with_stable_semantics_is_reviewable() {
    if !git_available() {
        return;
    }
    let fixture = fixture("producer");
    let lock_path = fixture.proposal.join("zrail.lock");
    let mut lock = LockFile::read(&lock_path).expect("read proposed lock");
    lock.producer = "0.0.2".into();
    lock.write(&lock_path).expect("write newer producer");

    let result = review(&options(&fixture)).expect("review newer producer");

    assert_eq!(result.exit_code, 0);
    reset(&fixture);
}

#[test]
fn old_proposed_semantics_fail_explicitly() {
    if !git_available() {
        return;
    }
    let fixture = fixture("old-semantics");
    let lock_path = fixture.proposal.join("zrail.lock");
    let mut lock = LockFile::read(&lock_path).expect("read proposed lock");
    lock.semantics = zrail_core::LOCK_SEMANTICS - 1;
    lock.write(&lock_path).expect("write old proposed lock");

    let result = review(&options(&fixture)).expect("review old proposed lock");

    assert_eq!(result.exit_code, 1);
    assert!(result.text.contains("error[REVIEW-004]"));
    assert!(result.text.contains("current semantics"));
    reset(&fixture);
}
