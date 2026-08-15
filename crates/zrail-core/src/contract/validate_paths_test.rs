//! Contract selectors reject patterns that exceed matcher safety limits.

use crate::path::{MAX_GLOB_PATTERN_BYTES, MAX_GLOB_PATTERN_SEGMENTS};

use super::{validate_package_pattern, validate_repository_pattern};
use crate::contract::validate_limits::ValidationErrors;

#[test]
fn repository_patterns_have_byte_and_segment_limits() {
    let mut bytes = ValidationErrors::new();
    validate_repository_pattern(&"x".repeat(MAX_GLOB_PATTERN_BYTES + 1), &mut bytes);
    assert!(bytes.finish().iter().any(|error| error.contains("byte")));

    let mut segments = ValidationErrors::new();
    let pattern = std::iter::repeat_n("x", MAX_GLOB_PATTERN_SEGMENTS + 1)
        .collect::<Vec<_>>()
        .join("/");
    validate_repository_pattern(&pattern, &mut segments);
    assert!(
        segments
            .finish()
            .iter()
            .any(|error| error.contains("segment"))
    );
}

#[test]
fn package_patterns_have_a_byte_limit() {
    let mut errors = ValidationErrors::new();
    validate_package_pattern(&"x".repeat(MAX_GLOB_PATTERN_BYTES + 1), &mut errors);
    assert!(!errors.finish().is_empty());
}
