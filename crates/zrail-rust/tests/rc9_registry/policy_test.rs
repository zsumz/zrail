//! Full original typed parsing and independent native schema validation.

#[path = "model.rs"]
mod model;
#[path = "original.rs"]
mod original;
#[path = "projection.rs"]
mod projection;

#[test]
fn registry_original_types_and_loader_are_exact_frozen_excerpts() {
    model::source_binding();
}

#[test]
fn registry_complete_original_matches_typed_conversion_stages() {
    model::source_cases();
}

#[test]
fn registry_native_schema_is_independent_and_strict() {
    model::native_cases();
}
