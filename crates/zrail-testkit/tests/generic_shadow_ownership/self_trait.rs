//! `Self` keeps concrete, trait, and projected identities distinct.

use super::fixture::{Repository, call_owner, capability_owner, exact_owner_count, finding_count};

#[test]
fn trait_impl_self_call_reaches_trait_owner() {
    let repository = Repository::new(
        "trait-impl-self-call",
        "mod owner; pub trait Launch { fn launch(); } pub struct Process; impl Launch for Process { fn launch() { Self::launch(); } }",
        "pub fn own() { <crate::Process as crate::Launch>::launch(); }",
        &call_owner("launch", "crate::Launch::launch"),
    );
    assert_resolution(&repository, "src/lib.rs", 1);
}

#[test]
fn trait_impl_self_reference_reaches_trait_capability_owner() {
    let repository = Repository::new(
        "trait-impl-self-reference",
        "mod owner; pub trait Launch { fn launch(); } pub struct Process; impl Launch for Process { fn launch() { let _ = Self::launch; } }",
        "pub fn own() { let _ = <crate::Process as crate::Launch>::launch; }",
        &capability_owner("launch", "crate::Launch::launch"),
    );
    assert_resolution(&repository, "src/lib.rs", 1);
}

#[test]
fn trait_default_self_call_reaches_current_trait_owner() {
    let repository = Repository::new(
        "trait-default-self-call",
        "mod owner; pub trait Launch { fn launch() { Self::launch(); } } pub struct Process; impl Launch for Process {}",
        "pub fn own() { <crate::Process as crate::Launch>::launch(); }",
        &call_owner("launch", "crate::Launch::launch"),
    );
    assert_resolution(&repository, "src/lib.rs", 1);
}

#[test]
fn trait_default_self_call_reaches_supertrait_provider() {
    let repository = Repository::new(
        "trait-default-supertrait",
        "mod owner; pub trait Base { fn ready(); } pub trait Derived: Base { fn run() { Self::ready(); } } pub struct Process; impl Base for Process { fn ready() {} } impl Derived for Process {}",
        "pub fn own() { <crate::Process as crate::Base>::ready(); }",
        &call_owner("base-ready", "crate::Base::ready"),
    );
    assert_resolution(&repository, "src/lib.rs", 1);
}

#[test]
fn self_associated_type_call_is_not_exact_concrete_path() {
    let repository = Repository::new(
        "self-associated-projection",
        "mod owner; pub trait Factory { fn ready(); } pub trait Provider where Self::Factory: Factory { type Factory; fn run() { Self::Factory::ready(); } } pub struct Process;",
        "pub fn own() { crate::Process::Factory::ready(); }",
        &call_owner("false-concrete", "crate::Process::Factory::ready"),
    );
    let report = repository.check();
    assert_eq!(
        exact_owner_count(&report, "false-concrete", "src/lib.rs"),
        0
    );
}

#[test]
fn generic_associated_type_bound_reaches_provider_trait() {
    let repository = projected_repository(
        "generic-associated-provider",
        "pub fn run<T: Provider>() where T::Factory: Factory { T::Factory::ready(); }",
    );
    assert_resolution(&repository, "src/lib.rs", 1);
}

#[test]
fn unbounded_generic_projection_is_not_global_incompleteness() {
    let repository = Repository::new(
        "unbounded-generic-projection",
        "mod owner; pub trait Provider { type Error; } pub fn run<D: Provider>() { D::Error::custom(); }",
        "",
        "",
    );
    assert_resolution(&repository, "src/lib.rs", 0);
}

#[test]
fn included_self_projection_matches_direct_projection() {
    let repository = Repository::new(
        "included-associated-provider",
        "mod owner; pub trait Factory { fn ready(); } pub trait Provider where Self::Factory: Factory { type Factory; fn direct() { Self::Factory::ready(); } fn included() { include!(\"expr.rs\"); } } pub struct Product; impl Factory for Product { fn ready() {} }",
        "pub fn own() { <crate::Product as crate::Factory>::ready(); }",
        &call_owner("factory-ready", "crate::Factory::ready"),
    );
    repository.write("src/expr.rs", "{ Self::Factory::ready(); }");
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

fn projected_repository(name: &str, body: &str) -> Repository {
    Repository::new(
        name,
        &format!(
            "mod owner; pub trait Factory {{ fn ready(); }} pub trait Provider {{ type Factory; }} pub struct Product; impl Factory for Product {{ fn ready() {{}} }} {body}"
        ),
        "pub fn own() { <crate::Product as crate::Factory>::ready(); }",
        &call_owner("factory-ready", "crate::Factory::ready"),
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
