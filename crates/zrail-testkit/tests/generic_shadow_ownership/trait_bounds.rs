//! Generic bounds retain lexical trait identity and plausible providers.

use super::fixture::{
    Repository, call_owner, conservative_owner_count, exact_owner_count, finding_count,
};

#[test]
fn imported_trait_bound_reaches_defining_owner() {
    let repository = repository(
        "imported-trait-bound",
        "use api::Factory; fn run<T: Factory>() { T::ready(); }",
    );
    assert_resolution(&repository, "src/lib.rs", 1);
}

#[test]
fn aliased_trait_bound_reaches_defining_owner() {
    let repository = repository(
        "aliased-trait-bound",
        "use api::Factory as F; fn run<T: F>() { T::ready(); }",
    );
    assert_resolution(&repository, "src/lib.rs", 1);
}

#[test]
fn nested_module_trait_bound_is_module_qualified() {
    let repository = repository(
        "nested-trait-bound",
        "mod nested { use crate::api::Factory; pub fn run<T: Factory>() { T::ready(); } }",
    );
    assert_resolution(&repository, "src/lib.rs", 1);
}

#[test]
fn glob_import_trait_bound_reaches_defining_owner() {
    let repository = repository(
        "glob-trait-bound",
        "use api::*; fn run<T: Factory>() { T::ready(); }",
    );
    assert_resolution(&repository, "src/lib.rs", 1);
}

#[test]
fn glob_supertrait_provider_preserves_conservative_quality() {
    let repository = Repository::new(
        "glob-supertrait-quality",
        "mod owner; mod api { pub trait Base { fn ready(); } } use api::*; pub trait Derived: Base {} pub struct Worker; impl Base for Worker { fn ready() {} } impl Derived for Worker {} pub fn run<T: Derived>() { T::ready(); }",
        "pub fn own() { <crate::Worker as crate::api::Base>::ready(); }",
        &call_owner("base-ready", "crate::api::Base::ready"),
    );
    let report = repository.check();
    assert!(
        conservative_owner_count(&report, "base-ready", "src/lib.rs") > 0,
        "{}",
        report.human()
    );
    assert_eq!(exact_owner_count(&report, "base-ready", "src/lib.rs"), 0);
}

#[test]
fn supertrait_associated_call_reaches_provider_owner() {
    let repository = Repository::new(
        "supertrait-provider",
        "mod owner; pub trait Base { fn ready(); } pub trait Derived: Base {} pub struct Worker; impl Base for Worker { fn ready() {} } impl Derived for Worker {} pub fn run<T: Derived>() { T::ready(); }",
        "pub fn own() { <crate::Worker as crate::Base>::ready(); }",
        &call_owner("base-ready", "crate::Base::ready"),
    );
    assert_resolution(&repository, "src/lib.rs", 1);
}

#[test]
fn cross_file_supertrait_reaches_provider_owner() {
    let repository = Repository::new(
        "cross-file-supertrait",
        "mod api; mod owner; use api::Derived; pub fn run<T: Derived>() { T::ready(); }",
        "pub fn own() { <crate::api::Worker as crate::api::Base>::ready(); }",
        &call_owner("base-ready", "crate::api::Base::ready"),
    );
    repository.write(
        "src/api.rs",
        "pub trait Base { fn ready(); } pub trait Derived: Base {} pub struct Worker; impl Base for Worker { fn ready() {} } impl Derived for Worker {}",
    );
    assert_resolution(&repository, "src/lib.rs", 1);
}

#[test]
fn transitive_supertrait_reaches_defining_provider() {
    let repository = Repository::new(
        "transitive-supertrait",
        "mod owner; pub trait Base { fn ready(); } pub trait Middle: Base {} pub trait Derived: Middle {} pub struct Worker; impl Base for Worker { fn ready() {} } impl Middle for Worker {} impl Derived for Worker {} pub fn run<T: Derived>() { T::ready(); }",
        "pub fn own() { <crate::Worker as crate::Base>::ready(); }",
        &call_owner("base-ready", "crate::Base::ready"),
    );
    assert_resolution(&repository, "src/lib.rs", 1);
}

#[test]
fn unrelated_import_alias_does_not_match() {
    let repository = repository(
        "unrelated-import-alias",
        "use api::Factory as F; trait Other { fn ready(); } fn run<T: Other>() { T::ready(); }",
    );
    assert_resolution(&repository, "src/lib.rs", 0);
}

#[test]
fn direct_and_included_alias_bounds_have_identical_candidates() {
    let repository = repository(
        "included-alias-bound",
        "use api::Factory as F; fn direct<T: F>() { T::ready(); } fn included<T: F>() { include!(\"expr.rs\"); }",
    );
    repository.write("src/expr.rs", "{ T::ready(); }");
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

fn repository(name: &str, body: &str) -> Repository {
    Repository::new(
        name,
        &format!(
            "mod owner; mod api {{ pub trait Factory {{ fn ready(); }} pub struct Worker; impl Factory for Worker {{ fn ready() {{}} }} }} {body}"
        ),
        "pub fn own() { <crate::api::Worker as crate::api::Factory>::ready(); }",
        &call_owner("factory-ready", "crate::api::Factory::ready"),
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
