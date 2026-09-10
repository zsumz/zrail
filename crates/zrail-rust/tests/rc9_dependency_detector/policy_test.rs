//! Exact raw-line predicates qualify detector assertions, not package graph authority.

#[path = "model.rs"]
mod model;
#[path = "original_test.rs"]
mod original;
#[path = "qualification.rs"]
mod qualification;

#[test]
fn dependency_detector_original_excerpts_are_bound() {
    model::source_binding();
}

#[test]
fn dependency_detector_lines_match_complete_original_name_sets() {
    original::check();
    model::qualify();
}

#[test]
#[ignore = "requires the clean committed producer and frozen snapshots"]
fn qualify_frozen_dependency_detector() {
    qualification::run();
}
