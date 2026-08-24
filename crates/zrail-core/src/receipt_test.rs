//! Strict receipt parsing and deterministic input binding.

use super::*;

const DIGEST: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

#[test]
fn accepts_a_versioned_receipt_and_binds_every_mirror_input() {
    let receipt = parse_execution_receipt(&format!(
        r#"{{"schema":1,"producer":"runner 1.2.3","input_sha256":"{DIGEST}","toolchain":"1.90.0","tests":[{{"id":"state_transitions","status":"passed"}}]}}"#
    ))
    .expect("valid receipt");
    assert_eq!(receipt.tests[0].status, ExecutionReceiptStatus::Passed);

    let digest =
        test_mirror_input_sha256("src/state.rs", b"one", "tests/state.rs", b"two", "works");
    assert_ne!(
        digest,
        test_mirror_input_sha256(
            "src/state.rs",
            b"changed",
            "tests/state.rs",
            b"two",
            "works"
        )
    );
    assert_ne!(
        digest,
        test_mirror_input_sha256("src/state.rs", b"one", "tests/state.rs", b"two", "other")
    );
}

#[test]
fn rejects_unversioned_producers_bad_digests_and_duplicate_tests() {
    for source in [
        format!(
            r#"{{"schema":2,"producer":"runner 1.2.3","input_sha256":"{DIGEST}","tests":[{{"id":"works","status":"passed"}}]}}"#
        ),
        format!(
            r#"{{"schema":1,"producer":"runner","input_sha256":"{DIGEST}","tests":[{{"id":"works","status":"passed"}}]}}"#
        ),
        r#"{"schema":1,"producer":"runner 1.2.3","input_sha256":"ABC","tests":[{"id":"works","status":"passed"}]}"#.into(),
        format!(
            r#"{{"schema":1,"producer":"runner 1.2.3","input_sha256":"{DIGEST}","tests":[{{"id":"works","status":"passed"}},{{"id":"works","status":"failed"}}]}}"#
        ),
    ] {
        assert!(parse_execution_receipt(&source).is_err());
    }
}
