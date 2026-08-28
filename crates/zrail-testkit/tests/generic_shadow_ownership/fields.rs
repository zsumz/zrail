//! Typed generic receivers never borrow a same-named outer field declaration.

use super::fixture::{Repository, exact_owner_count, field_owner, unresolved_owner_count};

#[test]
fn generic_deref_receiver_does_not_borrow_outer_field_identity() {
    let repository = Repository::new(
        "generic-deref-outer",
        READ_SOURCE,
        STATE_READ_OWNER,
        &field_owner("state-read", "field-read", "crate::State::secret"),
    );
    let report = repository.check();
    assert_eq!(exact_owner_count(&report, "state-read", "src/lib.rs"), 0);
}

#[test]
fn generic_deref_receiver_reaches_actual_field_owner_or_fails_closed() {
    let repository = Repository::new(
        "generic-deref-actual",
        READ_SOURCE,
        ACTUAL_READ_OWNER,
        &field_owner("actual-read", "field-read", "crate::Actual::secret"),
    );
    let report = repository.check();
    assert_eq!(
        unresolved_owner_count(&report, "actual-read", "src/lib.rs"),
        1,
        "{}",
        report.human()
    );
}

#[test]
fn generic_derefmut_write_does_not_borrow_outer_field_identity() {
    let repository = Repository::new(
        "generic-derefmut-write",
        WRITE_SOURCE,
        STATE_WRITE_OWNER,
        &field_owner("state-write", "field-write", "crate::State::secret"),
    );
    let report = repository.check();
    assert_eq!(exact_owner_count(&report, "state-write", "src/lib.rs"), 0);
    assert_eq!(
        unresolved_owner_count(&report, "state-write", "src/lib.rs"),
        1,
        "{}",
        report.human()
    );
}

#[test]
fn generic_derefmut_borrow_does_not_borrow_outer_field_identity() {
    let repository = Repository::new(
        "generic-derefmut-borrow",
        BORROW_SOURCE,
        STATE_BORROW_OWNER,
        &field_owner(
            "state-borrow",
            "field-mutable-borrow",
            "crate::State::secret",
        ),
    );
    let report = repository.check();
    assert_eq!(exact_owner_count(&report, "state-borrow", "src/lib.rs"), 0);
    assert_eq!(
        unresolved_owner_count(&report, "state-borrow", "src/lib.rs"),
        1,
        "{}",
        report.human()
    );
}

#[test]
fn generic_receiver_and_outer_type_sharing_field_name_remain_distinct() {
    let repository = Repository::new(
        "generic-deref-distinct",
        READ_SOURCE,
        STATE_READ_OWNER,
        &field_owner("state-read", "field-read", "crate::State::secret"),
    );
    let report = repository.check();
    assert_eq!(exact_owner_count(&report, "state-read", "src/lib.rs"), 0);
    assert_eq!(
        unresolved_owner_count(&report, "state-read", "src/lib.rs"),
        1,
        "{}",
        report.human()
    );
}

const READ_SOURCE: &str = "mod owner; use core::ops::Deref; pub struct State { secret: u64 } pub struct Actual { secret: u64 } pub fn trespass<State>(state: State) -> u64 where State: Deref<Target = Actual> { state.secret }";
const WRITE_SOURCE: &str = "mod owner; use core::ops::DerefMut; pub struct State { secret: u64 } pub struct Actual { secret: u64 } pub fn trespass<State>(mut state: State) where State: DerefMut<Target = Actual> { state.secret = 1; }";
const BORROW_SOURCE: &str = "mod owner; use core::ops::DerefMut; pub struct State { secret: u64 } pub struct Actual { secret: u64 } pub fn trespass<State>(mut state: State) where State: DerefMut<Target = Actual> { let _ = &mut state.secret; }";
const STATE_READ_OWNER: &str = "pub fn own(state: crate::State) -> u64 { state.secret }";
const ACTUAL_READ_OWNER: &str = "pub fn own(actual: crate::Actual) -> u64 { actual.secret }";
const STATE_WRITE_OWNER: &str = "pub fn own(mut state: crate::State) { state.secret = 1; }";
const STATE_BORROW_OWNER: &str =
    "pub fn own(mut state: crate::State) { let _ = &mut state.secret; }";
