//! Version-prefix translation does not grant ranges native matching semantics.

#[path = "matrix.rs"]
mod matrix;
#[path = "model.rs"]
mod model;
#[path = "native.rs"]
mod native;
#[path = "original.rs"]
mod original;
#[path = "qualification.rs"]
mod qualification;

#[test]
fn lock_prefix_original_operation_is_an_exact_frozen_excerpt() {
    model::source_binding();
}
#[test]
fn lock_prefix_pairs_distinguish_source_and_native_representations() {
    assert_eq!(matrix::run().len(), 120);
}
#[test]
fn lock_prefix_native_versions_remain_literal_not_semver_ranges() {
    assert_eq!(
        matrix::literals(
            &model::project().join("crates/zrail-testkit/tests/fixtures/rc9/lock-packages/valid")
        )
        .len(),
        20
    );
}
#[test]
fn lock_prefix_complete_original_and_native_accept_frozen_inputs() {
    model::frozen(
        &model::project().join("crates/zrail-testkit/tests/fixtures/rc9/lock-packages/valid"),
    );
}
#[test]
#[ignore = "requires frozen snapshots and a fresh ZRAIL_RC9_LOCK_PREFIX_REPORT"]
fn qualify_frozen_lock_prefix() {
    qualification::run();
}
