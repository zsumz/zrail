//! Calls and capability paths retain generic identity beyond construction policy.

use super::fixture::{
    Repository, assert_no_owner, call_owner, capability_owner, construction_owner,
    exact_owner_count, finding_count,
};

#[test]
fn generic_trait_call_reaches_trait_owner_or_fails_closed() {
    let repository = Repository::new(
        "generic-trait-owner",
        CALL_SOURCE,
        TRAIT_OWNER_SOURCE,
        &call_owner("factory-ready", "crate::Factory::ready"),
    );
    let report = repository.check();
    assert_eq!(
        finding_count(
            &report,
            "RUST-CALL-001",
            "rust.source.call-resolution",
            "src/lib.rs",
        ),
        1,
        "{}",
        report.human()
    );
}

#[test]
fn generic_trait_call_does_not_reach_same_named_outer_inherent_owner() {
    let repository = Repository::new(
        "generic-outer-inherent",
        CALL_SOURCE,
        "pub fn own() { crate::Choice::ready(); }",
        &call_owner("choice-ready", "crate::Choice::ready"),
    );
    let report = repository.check();
    assert_no_owner(&report, "choice-ready", "src/lib.rs");
}

#[test]
fn generic_trait_call_does_not_reach_same_named_outer_variant_owner() {
    let repository = Repository::new(
        "generic-outer-variant",
        "mod owner; pub enum Choice { Ready(u64) } pub trait Factory { #[allow(non_snake_case)] fn Ready(value: u64) -> Self; } pub fn trespass<Choice: Factory>() -> Choice { Choice::Ready(1) }",
        "pub fn own() -> crate::Choice { crate::Choice::Ready(1) }",
        &construction_owner("choice-ready", "crate::Choice"),
    );
    let report = repository.check();
    assert_eq!(exact_owner_count(&report, "choice-ready", "src/lib.rs"), 0);
}

#[test]
fn const_generic_reference_does_not_reach_outer_capability_owner() {
    let repository = Repository::new(
        "const-capability",
        CONST_SOURCE,
        "pub fn own() -> crate::Marker { crate::Marker }",
        &capability_owner("marker-capability", "crate::Marker"),
    );
    let report = repository.check();
    assert_eq!(
        exact_owner_count(&report, "marker-capability", "src/lib.rs"),
        1,
        "{}",
        report.human()
    );
}

#[test]
fn const_generic_reference_does_not_reach_outer_denied_symbol() {
    let repository = Repository::new(
        "const-denied-symbol",
        CONST_SOURCE,
        "",
        r#"[[scope]]
name = "marker-symbols"
include = ["src/lib.rs"]
reason = "The const parameter is not the outer unit struct."
[scope.symbols]
deny = ["crate::Marker"]
"#,
    );
    let report = repository.check();
    assert_eq!(
        finding_count(&report, "CAP-001", "marker-symbols", "src/lib.rs"),
        1,
        "{}",
        report.human()
    );
}

#[test]
fn included_generic_trait_call_matches_direct_generic_trait_call() {
    let repository = Repository::new(
        "included-generic-call",
        "mod owner; pub struct Choice; impl Choice { fn ready() {} } pub trait Factory { fn ready(); } pub struct Worker; impl Factory for Worker { fn ready() {} } pub fn direct<Choice: Factory>() { Choice::ready(); } pub fn included<Choice: Factory>() { include!(\"expr.rs\"); }",
        "pub fn own() { crate::Choice::ready(); }",
        &call_owner("choice-ready", "crate::Choice::ready"),
    );
    repository.write("src/expr.rs", "Choice::ready()");
    let report = repository.check();
    assert_no_owner(&report, "choice-ready", "src/lib.rs");
    assert_no_owner(&report, "choice-ready", "src/expr.rs");
    assert_eq!(
        resolution_count(&report, "src/lib.rs"),
        1,
        "{}",
        report.human()
    );
    assert_eq!(
        resolution_count(&report, "src/expr.rs"),
        1,
        "{}",
        report.human()
    );
}

fn resolution_count(report: &zrail_core::Report, path: &str) -> usize {
    finding_count(report, "RUST-CALL-001", "rust.source.call-resolution", path)
}

const CALL_SOURCE: &str = "mod owner; pub struct Choice; impl Choice { fn ready() {} } pub trait Factory { fn ready(); } pub struct Worker; impl Factory for Worker { fn ready() {} } pub fn trespass<Choice: Factory>() { Choice::ready(); }";
const TRAIT_OWNER_SOURCE: &str = "pub fn own() { <crate::Worker as crate::Factory>::ready(); }";
const CONST_SOURCE: &str = "mod owner; pub struct Marker; #[allow(non_upper_case_globals)] pub fn read<const Marker: usize>() -> usize { Marker }";
