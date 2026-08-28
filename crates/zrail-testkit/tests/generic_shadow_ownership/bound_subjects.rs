//! Equivalent Rust bound subjects produce the same trait-provider authority.

use super::fixture::{Repository, call_owner, finding_count};

#[test]
fn associated_type_declaration_bound_reaches_provider_owner() {
    let repository = associated_repository(
        "associated-declaration-bound",
        "pub trait Provider { type Factory: Factory; } pub fn run<T: Provider>() { T::Factory::ready(); }",
    );

    assert_resolution(&repository, "src/lib.rs", 1);
}

#[test]
fn trait_default_associated_type_declaration_bound_reaches_provider_owner() {
    let repository = associated_repository(
        "trait-default-associated-bound",
        "pub trait Provider { type Factory: Factory; fn run() { Self::Factory::ready(); } }",
    );

    assert_resolution(&repository, "src/lib.rs", 1);
}

#[test]
fn where_self_supertrait_reaches_provider_owner() {
    let repository = supertrait_repository(
        "where-self-supertrait",
        "pub trait Derived where Self: Base {} pub fn run<T: Derived>() { T::ready(); }",
    );

    assert_resolution(&repository, "src/lib.rs", 1);
}

#[test]
fn qualified_projection_bound_reaches_provider_owner() {
    let repository = associated_repository(
        "qualified-projection-bound",
        "pub trait Provider { type Factory; } pub fn run<T: Provider>() where <T as Provider>::Factory: Factory { <T as Provider>::Factory::ready(); }",
    );

    assert_resolution(&repository, "src/lib.rs", 1);
}

#[test]
fn colon_and_where_self_supertraits_have_identical_candidates() {
    let repository = supertrait_repository(
        "equivalent-supertrait-spellings",
        "pub trait Colon: Base {} pub trait Where where Self: Base {} pub fn colon<T: Colon>() { T::ready(); } pub fn where_form<T: Where>() { T::ready(); }",
    );

    assert_resolution(&repository, "src/lib.rs", 2);
}

#[test]
fn aliased_associated_type_declaration_bound_is_canonical() {
    let repository = Repository::new(
        "aliased-associated-declaration-bound",
        "mod owner; mod api { pub trait Factory { fn ready(); } pub struct Product; impl Factory for Product { fn ready() {} } } use api::Factory as F; pub trait Provider { type Factory: F; } pub fn run<T: Provider>() { T::Factory::ready(); }",
        "pub fn own() { <crate::api::Product as crate::api::Factory>::ready(); }",
        &call_owner("factory-ready", "crate::api::Factory::ready"),
    );

    assert_resolution(&repository, "src/lib.rs", 1);
}

#[test]
fn external_provider_closure_blocks_matching_owner() {
    let repository = external_repository(
        "external-provider-owner",
        "pub fn own() { dependency::Base::ready(); }",
        &call_owner("external-base-ready", "dependency::Base::ready"),
    );

    assert_resolution(&repository, "src/lib.rs", 1);
}

#[test]
fn external_provider_closure_blocks_matching_prefix_owner() {
    let repository = external_repository(
        "external-provider-prefix-owner",
        "pub fn own() { dependency::Base::ready(); }",
        &call_owner("external-base", "dependency::Base"),
    );

    assert_resolution(&repository, "src/lib.rs", 1);
}

#[test]
fn external_provider_closure_blocks_matching_denied_scope() {
    let repository = external_repository(
        "external-provider-scope",
        "",
        r#"[[scope]]
name = "external-base-ready"
include = ["src/lib.rs"]
reason = "Unknown external providers must not bypass denied authority."
[scope.symbols]
deny = ["dependency::Base::ready"]
"#,
    );

    assert_resolution(&repository, "src/lib.rs", 1);
}

#[test]
fn external_provider_closure_blocks_matching_prefix_scope() {
    let repository = external_repository(
        "external-provider-prefix-scope",
        "",
        r#"[[scope]]
name = "external-base"
include = ["src/lib.rs"]
reason = "Unknown external providers must not bypass prefix authority."
[scope.symbols]
deny = ["dependency::Base"]
"#,
    );

    assert_resolution(&repository, "src/lib.rs", 1);
}

#[test]
fn external_provider_closure_blocks_cross_authority_until_attested() {
    let repository = external_repository(
        "external-provider-unrelated",
        "pub fn own() { other::Base::ready(); }",
        &call_owner("other-base-ready", "other::Base::ready"),
    );

    assert_resolution(&repository, "src/lib.rs", 1);
}

#[test]
fn external_provider_closure_ignores_unrelated_associated_items() {
    let repository = external_repository(
        "external-provider-unrelated-item",
        "pub fn own() { let _ = std::process::Command::new(\"echo\"); }",
        &call_owner("process-new", "std::process::Command::new"),
    );

    assert_resolution(&repository, "src/lib.rs", 0);
}

#[test]
fn opaque_local_provider_closure_blocks_matching_prefix_owner() {
    let repository = Repository::new(
        "opaque-local-provider-prefix",
        "mod owner; pub trait Base { fn ready(); } #[provider] pub trait Derived {} pub struct Worker; impl Base for Worker { fn ready() {} } pub fn run<T: Derived>() { T::ready(); }",
        "pub fn own() { <crate::Worker as crate::Base>::ready(); }",
        &call_owner("local-base", "crate::Base"),
    );

    assert_resolution(&repository, "src/lib.rs", 1);
}

#[test]
fn opaque_local_provider_closure_blocks_bare_local_owner() {
    let repository = Repository::new(
        "opaque-local-provider-bare",
        "mod owner; pub trait Base { fn ready(); } #[provider] pub trait Derived {} pub struct Worker; impl Base for Worker { fn ready() {} } pub fn run<T: Derived>() { T::ready(); }",
        "pub fn own() { <crate::Worker as crate::Base>::ready(); }",
        &call_owner("local-base-ready", "Base::ready"),
    );

    assert_resolution(&repository, "src/lib.rs", 1);
}

#[test]
fn opaque_associated_type_surface_blocks_provider_owner() {
    let repository = Repository::new(
        "opaque-associated-type",
        "mod owner; pub trait Base { fn ready(); } pub trait Provider { #[provider] type Factory; } pub struct Product; impl Base for Product { fn ready() {} } pub fn run<T: Provider>() { T::Factory::ready(); }",
        "pub fn own() { <crate::Product as crate::Base>::ready(); }",
        &call_owner("base-ready", "crate::Base::ready"),
    );

    assert_resolution(&repository, "src/lib.rs", 1);
}

#[test]
fn opaque_associated_type_preserves_explicit_and_possible_providers() {
    let repository = Repository::new(
        "opaque-associated-type-explicit",
        "mod owner; pub trait Base { fn ready(); } pub trait Other { fn ready(); } pub trait Provider { #[provider] type Factory: Base; } pub struct Product; impl Other for Product { fn ready() {} } pub fn run<T: Provider>() { T::Factory::ready(); }",
        "pub fn own() { <crate::Product as crate::Other>::ready(); }",
        &call_owner("other-ready", "crate::Other::ready"),
    );

    assert_resolution(&repository, "src/lib.rs", 1);
}

#[test]
fn trait_body_macro_keeps_unknown_projection_surface_incomplete() {
    let repository = Repository::new(
        "trait-body-macro-projection",
        "mod owner; pub trait Base { fn ready(); } pub trait Provider { provider!(); } pub struct Product; impl Base for Product { fn ready() {} } pub fn run<T: Provider>() { T::Factory::ready(); }",
        "pub fn own() { <crate::Product as crate::Base>::ready(); }",
        &call_owner("base-ready", "crate::Base::ready"),
    );

    assert_resolution(&repository, "src/lib.rs", 1);
}

#[test]
fn direct_and_inherited_projection_bounds_are_additive() {
    let repository = Repository::new(
        "additive-projection-bounds",
        "mod owner; pub trait Base { fn ready(); } pub trait Marker {} pub trait Provider { type Factory: Base; } pub struct Product; impl Base for Product { fn ready() {} } pub fn run<T: Provider>() where T::Factory: Marker { T::Factory::ready(); }",
        "pub fn own() { <crate::Product as crate::Base>::ready(); }",
        &call_owner("base-ready", "crate::Base::ready"),
    );

    assert_resolution(&repository, "src/lib.rs", 1);
}

#[test]
fn unresolved_supertrait_alias_keeps_base_closure_incomplete() {
    let repository = Repository::new(
        "unresolved-supertrait-alias",
        "mod owner; pub trait Base { fn ready(); } #[provider] use Base as Alias; pub trait Derived: Alias {} pub struct Worker; impl Base for Worker { fn ready() {} } pub fn run<T: Derived>() { T::ready(); }",
        "pub fn own() { <crate::Worker as crate::Base>::ready(); }",
        &call_owner("base-ready", "crate::Base::ready"),
    );

    assert_resolution(&repository, "src/lib.rs", 1);
}

fn associated_repository(name: &str, body: &str) -> Repository {
    Repository::new(
        name,
        &format!(
            "mod owner; pub trait Factory {{ fn ready(); }} pub struct Product; impl Factory for Product {{ fn ready() {{}} }} {body}"
        ),
        "pub fn own() { <crate::Product as crate::Factory>::ready(); }",
        &call_owner("factory-ready", "crate::Factory::ready"),
    )
}

fn supertrait_repository(name: &str, body: &str) -> Repository {
    Repository::new(
        name,
        &format!(
            "mod owner; pub trait Base {{ fn ready(); }} pub struct Worker; impl Base for Worker {{ fn ready() {{}} }} {body}"
        ),
        "pub fn own() { <crate::Worker as crate::Base>::ready(); }",
        &call_owner("base-ready", "crate::Base::ready"),
    )
}

fn external_repository(name: &str, owner_source: &str, contract: &str) -> Repository {
    let repository = Repository::new(
        name,
        "mod owner; use dependency::Derived; pub fn run<T: Derived>() { T::ready(); }",
        owner_source,
        contract,
    );
    repository.write(
        "Cargo.toml",
        r#"[package]
name = "generic-shadow-fixture"
version = "0.0.0"
edition = "2024"

[dependencies]
dependency = { package = "dependency", version = "1" }
other = { package = "other", version = "1" }
"#,
    );
    repository
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
