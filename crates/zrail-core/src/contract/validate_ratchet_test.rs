//! Ratchet rule vocabulary remains closed and explicit.

use crate::contract::TestMode;

use super::{compatible_with_test_mode, supported_rule};

#[test]
fn inline_tests_are_supported_without_opening_extensible_rule_names() {
    assert!(supported_rule("rust.file-size"));
    assert!(supported_rule("rust.inline-tests"));
    assert!(!supported_rule("rust.any-future-debt"));
    assert!(compatible_with_test_mode(
        "rust.inline-tests",
        TestMode::Sibling
    ));
    assert!(!compatible_with_test_mode(
        "rust.inline-tests",
        TestMode::Allow
    ));
}
