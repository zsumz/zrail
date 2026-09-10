//! Exact name-indexing failures and existing stricter native names remain explicit.

#[path = "model.rs"]
mod model;
#[path = "mutations.rs"]
mod mutations;
#[path = "original.rs"]
mod original;
#[path = "qualification.rs"]
mod qualification;

#[test]
fn lock_names_original_selection_is_an_exact_frozen_excerpt() {
    model::source_binding();
}

#[test]
fn lock_names_matrix_preserves_whole_array_selection_and_stricter_errors() {
    assert_eq!(model::matrix().len(), 32);
}

#[test]
fn lock_names_unrelated_mutations_execute_the_complete_original_guard() {
    assert_eq!(
        mutations::run(
            &model::project().join("crates/zrail-testkit/tests/fixtures/rc9/lock-packages/valid")
        )
        .len(),
        16
    );
}

#[test]
fn lock_names_complete_original_and_native_accept_frozen_inputs() {
    model::frozen(
        &model::project().join("crates/zrail-testkit/tests/fixtures/rc9/lock-packages/valid"),
    );
}

#[test]
#[ignore = "requires frozen snapshots and a fresh ZRAIL_RC9_LOCK_NAMES_REPORT"]
fn qualify_frozen_lock_names() {
    qualification::run();
}
