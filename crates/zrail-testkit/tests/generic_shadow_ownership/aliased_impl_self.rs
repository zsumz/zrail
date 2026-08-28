//! Implemented-trait aliases retain authority on concrete `Self` occurrences.

use super::fixture::{Repository, assert_no_owner, call_owner, capability_owner, finding_count};

#[test]
fn aliased_trait_impl_self_call_reaches_defining_trait_owner() {
    let repository = repository(
        "aliased-impl-self-call",
        "fn relay() { Self::launch(); }",
        &call_owner("launch", "crate::api::Launch::launch"),
    );

    assert_resolution(&repository, "src/lib.rs", 1);
}

#[test]
fn aliased_trait_impl_self_reference_reaches_trait_capability_owner() {
    let repository = repository(
        "aliased-impl-self-reference",
        "fn relay() { let _ = Self::launch; }",
        &capability_owner("launch", "crate::api::Launch::launch"),
    );

    assert_resolution(&repository, "src/lib.rs", 1);
}

#[test]
fn nested_module_trait_impl_self_call_is_canonical() {
    let repository = Repository::new(
        "nested-aliased-impl-self",
        "mod owner; mod api { pub trait Launch { fn launch(); fn relay(); } } mod nested { use crate::api::Launch as L; pub struct Process; impl L for Process { fn launch() {} fn relay() { Self::launch(); } } }",
        "pub fn own() { <crate::nested::Process as crate::api::Launch>::launch(); }",
        &call_owner("launch", "crate::api::Launch::launch"),
    );

    assert_resolution(&repository, "src/lib.rs", 1);
}

#[test]
fn direct_and_included_aliased_impl_self_have_identical_candidates() {
    let repository = Repository::new(
        "included-aliased-impl-self",
        "mod owner; mod api { pub trait Launch { fn launch(); fn direct(); fn included(); } } use api::Launch as L; pub struct Process; impl L for Process { fn launch() {} fn direct() { Self::launch(); } include!(\"impl_items.inc\"); }",
        "pub fn own() { <crate::Process as crate::api::Launch>::launch(); }",
        &call_owner("launch", "crate::api::Launch::launch"),
    );
    repository.write("src/impl_items.inc", "fn included() { Self::launch(); }");

    let report = repository.check();
    assert_eq!(
        resolution_count(&report, "src/lib.rs"),
        1,
        "{}",
        report.human()
    );
    assert_eq!(
        resolution_count(&report, "src/impl_items.inc"),
        1,
        "{}",
        report.human()
    );
}

#[test]
fn aliased_impl_self_does_not_match_an_unrelated_trait() {
    let repository = Repository::new(
        "unrelated-aliased-impl-self",
        "mod owner; mod api { pub trait Launch { fn launch(); fn relay(); } pub trait Other { fn launch(); } } use api::Launch as L; pub struct Process; impl L for Process { fn launch() {} fn relay() { Self::launch(); } }",
        "pub fn own() { <crate::Process as crate::api::Other>::launch(); }",
        &call_owner("other-launch", "crate::api::Other::launch"),
    );

    let report = repository.check();
    assert_eq!(
        resolution_count(&report, "src/lib.rs"),
        0,
        "{}",
        report.human()
    );
    assert_no_owner(&report, "other-launch", "src/lib.rs");
}

fn repository(name: &str, impl_item: &str, contract: &str) -> Repository {
    Repository::new(
        name,
        &format!(
            "mod owner; mod api {{ pub trait Launch {{ fn launch(); fn relay(); }} }} use api::Launch as L; pub struct Process; impl L for Process {{ fn launch() {{}} {impl_item} }}"
        ),
        "pub fn own() { <crate::Process as crate::api::Launch>::launch(); }",
        contract,
    )
}

fn assert_resolution(repository: &Repository, path: &str, expected: usize) {
    let report = repository.check();
    assert_eq!(
        resolution_count(&report, path),
        expected,
        "{}",
        report.human()
    );
}

fn resolution_count(report: &zrail_core::Report, path: &str) -> usize {
    finding_count(report, "RUST-CALL-001", "rust.source.call-resolution", path)
}
