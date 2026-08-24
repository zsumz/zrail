//! Rust policy selectors use their semantic normalized identity.

use super::{ValidationErrors, rust_selectors};

#[test]
fn raw_identifier_spellings_cannot_duplicate_a_selector() {
    let mut errors = ValidationErrors::new();

    rust_selectors(
        "source.rust.hygiene.deny_methods",
        &["unwrap".into(), "r#unwrap".into()],
        &mut errors,
    );

    assert!(errors.finish().join("\n").contains("duplicate normalized"));
}
