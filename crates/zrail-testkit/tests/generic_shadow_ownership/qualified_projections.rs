//! Explicit projection qualifiers constrain provider and fallback relevance.

use super::fixture::{Repository, call_owner, finding_count};

const SOURCE: &str = "mod owner; pub trait Left { type Item; } pub trait Right { type Item; } pub trait LeftFactory { fn make(); } pub trait RightFactory { fn make(); } pub fn run<T>() where T: Left + Right, <T as Left>::Item: LeftFactory, <T as Right>::Item: RightFactory { <T as Left>::Item::make(); }";

#[test]
fn qualified_left_projection_reaches_only_left_provider() {
    assert_selector(
        "qualified-left-provider",
        SOURCE,
        "crate::LeftFactory::make",
        1,
    );
}

#[test]
fn same_named_associated_types_do_not_cross_contaminate() {
    assert_selector(
        "qualified-left-not-right",
        SOURCE,
        "crate::RightFactory::make",
        0,
    );
}

#[test]
fn qualified_right_projection_reaches_only_right_provider() {
    let source = SOURCE.replace("<T as Left>::Item::make();", "<T as Right>::Item::make();");
    assert_selector(
        "qualified-right-provider",
        &source,
        "crate::RightFactory::make",
        1,
    );
    assert_selector(
        "qualified-right-not-left",
        &source,
        "crate::LeftFactory::make",
        0,
    );
}

#[test]
fn included_qualified_projection_matches_direct_source() {
    let repository = Repository::new(
        "included-qualified-projection",
        "mod owner; pub trait Left { type Item; } pub trait Right { type Item; } pub trait LeftFactory { fn make(); } pub trait RightFactory { fn make(); } pub struct Scope<T>(T); impl<T> Scope<T> where T: Left + Right, <T as Left>::Item: LeftFactory, <T as Right>::Item: RightFactory { include!(\"impl_items.inc\"); }",
        "pub fn own() { crate::RightFactory::make(); }",
        &call_owner("selected", "crate::RightFactory::make"),
    );
    repository.write(
        "src/impl_items.inc",
        "fn run() { <T as Left>::Item::make(); }",
    );

    assert_resolution(&repository, "src/impl_items.inc", 0);
}

#[test]
fn generic_arguments_distinguish_projection_qualifiers() {
    let source = "mod owner; pub trait Provider<A> { type Item; } pub trait LeftFactory { fn make(); } pub trait RightFactory { fn make(); } pub struct A; pub struct B; pub fn run<T>() where T: Provider<A, Item: LeftFactory> + Provider<B, Item: RightFactory> { <T as Provider<A>>::Item::make(); }";
    assert_selector(
        "generic-argument-left",
        source,
        "crate::LeftFactory::make",
        1,
    );
    assert_selector(
        "generic-argument-not-right",
        source,
        "crate::RightFactory::make",
        0,
    );
}

#[test]
fn included_generic_arguments_preserve_qualifier_identity() {
    let source = "mod owner; pub trait Provider<A> { type Item; } pub trait LeftFactory { fn make(); } pub trait RightFactory { fn make(); } pub struct A; pub struct B; pub struct Scope<T>(T); impl<T> Scope<T> where T: Provider<A, Item: LeftFactory> + Provider<B, Item: RightFactory> { include!(\"impl_items.inc\"); }";
    let left = Repository::new(
        "included-generic-argument-left",
        source,
        "pub fn own() { crate::LeftFactory::make(); }",
        &call_owner("selected", "crate::LeftFactory::make"),
    );
    left.write(
        "src/impl_items.inc",
        "fn run() { <T as Provider<A>>::Item::make(); }",
    );
    let right = Repository::new(
        "included-generic-argument-right",
        source,
        "pub fn own() { crate::RightFactory::make(); }",
        &call_owner("selected", "crate::RightFactory::make"),
    );
    right.write(
        "src/impl_items.inc",
        "fn run() { <T as Provider<A>>::Item::make(); }",
    );

    assert_resolution(&left, "src/impl_items.inc", 1);
    assert_resolution(&right, "src/impl_items.inc", 0);
}

#[test]
fn qualified_projection_alias_is_canonical() {
    let source = "mod owner; pub trait Left { type Item; } pub trait Right { type Item; } pub trait LeftFactory { fn make(); } pub trait RightFactory { fn make(); } use Left as L; pub fn run<T>() where T: L + Right, <T as L>::Item: LeftFactory, <T as Right>::Item: RightFactory { <T as L>::Item::make(); }";
    assert_selector(
        "qualified-alias-left",
        source,
        "crate::LeftFactory::make",
        1,
    );
    assert_selector(
        "qualified-alias-not-right",
        source,
        "crate::RightFactory::make",
        0,
    );
}

#[test]
fn unqualified_unique_projection_remains_precise() {
    let source = "mod owner; pub trait Left { type Item; } pub trait LeftFactory { fn make(); } pub trait RightFactory { fn make(); } pub fn run<T>() where T: Left, T::Item: LeftFactory { T::Item::make(); }";
    assert_selector(
        "unqualified-unique-left",
        source,
        "crate::LeftFactory::make",
        1,
    );
    assert_selector(
        "unqualified-unique-not-right",
        source,
        "crate::RightFactory::make",
        0,
    );
}

#[test]
fn ambiguous_unqualified_projection_fails_closed() {
    let source = "mod owner; pub trait Left { type Item; } pub trait Right { type Item; } pub trait LeftFactory { fn make(); } pub trait RightFactory { fn make(); } pub fn run<T>() where T: Left + Right, <T as Left>::Item: LeftFactory, <T as Right>::Item: RightFactory { T::Item::make(); }";
    assert_selector(
        "unqualified-ambiguous-left",
        source,
        "crate::LeftFactory::make",
        1,
    );
    assert_selector(
        "unqualified-ambiguous-right",
        source,
        "crate::RightFactory::make",
        1,
    );
}

#[test]
fn same_external_root_different_item_is_not_relevant() {
    let repository = external_repository(
        "external-different-item",
        "pub fn own() { dependency::Other::shutdown(); }",
        &call_owner("shutdown", "dependency::Other::shutdown"),
    );

    assert_resolution(&repository, "src/lib.rs", 0);
}

#[test]
fn same_local_authority_different_item_is_not_relevant() {
    let repository = Repository::new(
        "local-different-item",
        "mod owner; pub trait Base { fn ready(); } #[provider] pub trait Derived {} pub struct Other; impl Other { pub fn shutdown() {} } pub fn run<T: Derived>() { T::ready(); }",
        "pub fn own() { crate::Other::shutdown(); }",
        &call_owner("shutdown", "crate::Other::shutdown"),
    );

    assert_resolution(&repository, "src/lib.rs", 0);
}

#[test]
fn unknown_provider_same_item_remains_fail_closed() {
    let repository = external_repository(
        "external-same-item",
        "pub fn own() { dependency::Other::ready(); }",
        &call_owner("ready", "dependency::Other::ready"),
    );

    assert_resolution(&repository, "src/lib.rs", 1);
}

#[test]
fn trait_prefix_owner_remains_conservative() {
    let repository = external_repository(
        "external-prefix",
        "pub fn own() { dependency::Base::ready(); }",
        &call_owner("base", "dependency::Base"),
    );

    assert_resolution(&repository, "src/lib.rs", 1);
}

fn assert_selector(name: &str, source: &str, selector: &str, expected: usize) {
    let owner_source = format!("pub fn own() {{ {selector}(); }}");
    let repository = Repository::new(
        name,
        source,
        &owner_source,
        &call_owner("selected", selector),
    );
    assert_resolution(&repository, "src/lib.rs", expected);
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
        "[package]\nname = \"generic-shadow-fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n[dependencies]\ndependency = { version = \"1\" }\n",
    );
    repository
}

fn assert_resolution(repository: &Repository, path: &str, expected: usize) {
    let report = repository.check();
    assert_eq!(
        finding_count(
            &report,
            "RUST-CALL-001",
            "rust.source.call-resolution",
            path,
        ),
        expected,
        "{}",
        report.human(),
    );
}
