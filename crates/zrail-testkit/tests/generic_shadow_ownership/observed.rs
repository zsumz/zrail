//! Bound-associated calls and capabilities retain only plausible trait identities.

use super::fixture::{
    Repository, assert_no_owner, call_owner, capability_owner, construction_owner,
    exact_owner_count, finding_count,
};

#[test]
fn generic_trait_call_reaches_candidate_trait_owner() {
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
fn generic_trait_function_reference_reaches_capability_owner() {
    let repository = Repository::new(
        "generic-trait-reference",
        REFERENCE_SOURCE,
        TRAIT_CAPABILITY_OWNER_SOURCE,
        &capability_owner("factory-ready-capability", "crate::Factory::ready"),
    );
    assert_eq!(resolution_count(&repository.check(), "src/lib.rs"), 1);
}

#[test]
fn generic_associated_const_reaches_capability_owner() {
    let repository = Repository::new(
        "generic-associated-const",
        "mod owner; pub trait Limits { const MAX: usize; } pub struct Worker; impl Limits for Worker { const MAX: usize = 8; } pub fn trespass<T: Limits>() { let _ = T::MAX; }",
        "pub fn own() { let _ = <crate::Worker as crate::Limits>::MAX; }",
        &capability_owner("limits-max-capability", "crate::Limits::MAX"),
    );
    assert_eq!(resolution_count(&repository.check(), "src/lib.rs"), 1);
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
fn multiple_trait_candidates_fail_closed_selectively() {
    let source = "mod owner; pub trait Factory { fn ready(); } pub trait OtherFactory { fn ready(); } pub struct Worker; impl Factory for Worker { fn ready() {} } impl OtherFactory for Worker { fn ready() {} } pub fn trespass<T>() where T: Factory + OtherFactory { T::ready(); }";
    let candidate = Repository::new(
        "multiple-candidate-traits",
        source,
        "pub fn own() { <crate::Worker as crate::OtherFactory>::ready(); }",
        &call_owner("other-factory-ready", "crate::OtherFactory::ready"),
    );
    assert_eq!(resolution_count(&candidate.check(), "src/lib.rs"), 1);

    let unrelated = Repository::new(
        "multiple-unrelated-type",
        source,
        "pub struct Choice; impl Choice { pub fn ready() {} } pub fn own() { Choice::ready(); }",
        &call_owner("choice-ready", "crate::owner::Choice::ready"),
    );
    assert_eq!(resolution_count(&unrelated.check(), "src/lib.rs"), 0);
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
fn direct_and_included_generic_capabilities_match() {
    let repository = Repository::new(
        "included-generic-capability",
        "mod owner; pub trait Factory { fn ready(); } pub struct Worker; impl Factory for Worker { fn ready() {} } pub fn direct<T: Factory>() { let _ = T::ready; } pub fn included<T: Factory>() { include!(\"expr.rs\"); }",
        TRAIT_CAPABILITY_OWNER_SOURCE,
        &capability_owner("factory-ready-capability", "crate::Factory::ready"),
    );
    repository.write("src/expr.rs", "{ let _ = T::ready; }");
    let report = repository.check();
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

#[test]
fn generic_trait_reference_reaches_exact_symbol_scope() {
    let repository = Repository::new(
        "generic-trait-scope",
        REFERENCE_SOURCE,
        "",
        r#"[[scope]]
name = "factory-symbols"
include = ["src/lib.rs"]
reason = "The trait capability is denied in this scope."
[scope.symbols]
deny = ["crate::Factory::ready"]
"#,
    );
    assert_eq!(resolution_count(&repository.check(), "src/lib.rs"), 1);
}

fn resolution_count(report: &zrail_core::Report, path: &str) -> usize {
    finding_count(report, "RUST-CALL-001", "rust.source.call-resolution", path)
}

const CALL_SOURCE: &str = "mod owner; pub struct Choice; impl Choice { fn ready() {} } pub trait Factory { fn ready(); } pub struct Worker; impl Factory for Worker { fn ready() {} } pub fn trespass<Choice: Factory>() { Choice::ready(); }";
const TRAIT_OWNER_SOURCE: &str = "pub fn own() { <crate::Worker as crate::Factory>::ready(); }";
const REFERENCE_SOURCE: &str = "mod owner; pub trait Factory { fn ready(); } pub struct Worker; impl Factory for Worker { fn ready() {} } pub fn trespass<T: Factory>() { let _ = T::ready; }";
const TRAIT_CAPABILITY_OWNER_SOURCE: &str =
    "pub fn own() { let _ = <crate::Worker as crate::Factory>::ready; }";
const CONST_SOURCE: &str = "mod owner; pub struct Marker; #[allow(non_upper_case_globals)] pub fn read<const Marker: usize>() -> usize { Marker }";
