//! Array shape, later count failures and full frozen helper execution are separate evidence.

#[path = "model.rs"]
mod model;
#[path = "original.rs"]
mod original;
#[path = "qualification.rs"]
mod qualification;

#[test]
fn lock_array_original_precondition_is_an_exact_frozen_excerpt() {
    model::source_binding();
}

#[test]
fn lock_array_type_matrix_preserves_failure_stages() {
    assert_eq!(model::matrix().len(), 24);
}

#[test]
fn lock_array_empty_type_passes_but_all_five_required_counts_fail() {
    assert_eq!(model::empty_counts().len(), 5);
}

#[test]
fn lock_array_complete_original_and_native_accept_frozen_inputs() {
    model::frozen(
        &model::project().join("crates/zrail-testkit/tests/fixtures/rc9/lock-packages/valid"),
    );
}

#[test]
#[ignore = "requires frozen snapshots and a fresh ZRAIL_RC9_LOCK_ARRAY_REPORT"]
fn qualify_frozen_lock_array() {
    qualification::run();
}
