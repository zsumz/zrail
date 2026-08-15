//! Evidence identity parser coverage.

use super::{EvidenceReference, parse_evidence_reference};

#[test]
fn parses_exact_supported_evidence() {
    assert_eq!(
        parse_evidence_reference("rust-test:src/worker_test.rs::works"),
        Ok(EvidenceReference::RustTest {
            path: "src/worker_test.rs",
            test: "works"
        })
    );
    assert_eq!(
        parse_evidence_reference("gate:check"),
        Ok(EvidenceReference::Gate { name: "check" })
    );
}

#[test]
fn rejects_ambiguous_or_extensible_spellings() {
    for invalid in [
        "rust-test:src/lib.rs",
        "rust-test:src/lib.rs::module::works",
        "gate:check me",
        "command:cargo test",
    ] {
        assert!(parse_evidence_reference(invalid).is_err(), "{invalid}");
    }
}
