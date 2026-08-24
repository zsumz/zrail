//! Selector normalization keeps ratchet identity semantic.

use super::normalize_ratchet_selector;

#[test]
fn raw_identifier_spelling_is_not_part_of_identity() {
    assert_eq!(
        normalize_ratchet_selector("r#crate::r#panic"),
        "crate::panic"
    );
}
