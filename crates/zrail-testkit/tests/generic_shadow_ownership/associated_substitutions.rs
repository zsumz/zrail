//! Associated constraints and definitions preserve their concrete policy identity.

use super::fixture::{Repository, call_owner, finding_count};

#[test]
fn trait_argument_associated_bound_reaches_provider_owner() {
    let fixture = repository(
        "trait-argument-bound",
        "pub fn run<T>() where T: Provider<Factory: Factory> { <T as Provider>::Factory::ready(); }",
        "pub fn own() { <crate::Product as crate::Factory>::ready(); }",
        &call_owner("factory-ready", "crate::Factory::ready"),
    );

    assert_resolution(&fixture, "src/lib.rs", 1);

    let unrelated = repository(
        "trait-argument-bound-unrelated",
        "pub fn run<T>() where T: Provider<Factory: Factory> { <T as Provider>::Factory::ready(); }",
        "pub fn own() { crate::OtherFactory::ready(); }",
        &call_owner("other-ready", "crate::OtherFactory::ready"),
    );
    assert_resolution(&unrelated, "src/lib.rs", 0);
}

#[test]
fn trait_argument_associated_equality_reaches_concrete_owner() {
    let fixture = repository(
        "trait-argument-equality",
        "pub fn run<T>() where T: Provider<Factory = Product> { <T as Provider>::Factory::ready(); }",
        "pub fn own() { crate::Product::ready(); }",
        &call_owner("product-ready", "crate::Product::ready"),
    );

    assert_resolution(&fixture, "src/lib.rs", 1);

    let unrelated = repository(
        "trait-argument-equality-unrelated",
        "pub fn run<T>() where T: Provider<Factory = Product> { <T as Provider>::Factory::ready(); }",
        "pub fn own() { crate::OtherProduct::ready(); }",
        &call_owner("other-ready", "crate::OtherProduct::ready"),
    );
    assert_resolution(&unrelated, "src/lib.rs", 0);
}

#[test]
fn impl_associated_type_definition_reaches_concrete_owner() {
    let repository = repository(
        "impl-associated-definition",
        "impl Provider for Process { type Factory = Product; fn run() { Self::Factory::ready(); } }",
        "pub fn own() { crate::Product::ready(); }",
        &call_owner("product-ready", "crate::Product::ready"),
    );

    assert_resolution(&repository, "src/lib.rs", 1);
}

#[test]
fn aliased_associated_equality_is_canonical() {
    let repository = repository(
        "aliased-associated-equality",
        "use Product as P; pub fn run<T>() where T: Provider<Factory = P> { <T as Provider>::Factory::ready(); }",
        "pub fn own() { crate::Product::ready(); }",
        &call_owner("product-ready", "crate::Product::ready"),
    );

    assert_resolution(&repository, "src/lib.rs", 1);
}

#[test]
fn qualified_associated_definition_is_preserved() {
    let repository = repository(
        "qualified-associated-definition",
        "impl Provider for Process { type Factory = Product; fn run() { <Self as Provider>::Factory::ready(); } }",
        "pub fn own() { crate::Product::ready(); }",
        &call_owner("product-ready", "crate::Product::ready"),
    );

    assert_resolution(&repository, "src/lib.rs", 1);
}

#[test]
fn included_impl_associated_definition_is_projected() {
    let repository = repository(
        "included-associated-definition",
        "impl Provider for Process { type Factory = Product; include!(\"impl_items.inc\"); }",
        "pub fn own() { crate::Product::ready(); }",
        &call_owner("product-ready", "crate::Product::ready"),
    );
    repository.write(
        "src/impl_items.inc",
        "fn run() { <Self as Provider>::Factory::ready(); }",
    );

    assert_resolution(&repository, "src/impl_items.inc", 1);
}

#[test]
fn cfg_partitioned_associated_definition_is_domain_exact() {
    let source = "#[cfg(not(test))] impl Provider for Process { type Factory = Product; fn run() { Self::Factory::ready(); } } #[cfg(test)] impl Provider for Process { type Factory = OtherProduct; fn run() { Self::Factory::ready(); } }";
    let product = repository(
        "cfg-associated-product",
        source,
        "pub fn own() { crate::Product::ready(); }",
        &call_owner("product-ready", "crate::Product::ready"),
    );
    let other = repository(
        "cfg-associated-other",
        source,
        "pub fn own() { crate::OtherProduct::ready(); }",
        &call_owner("other-ready", "crate::OtherProduct::ready"),
    );

    assert_resolution(&product, "src/lib.rs", 1);
    assert_resolution(&other, "src/lib.rs", 1);
}

#[test]
fn gat_associated_definition_preserves_arguments() {
    let repository = Repository::new(
        "gat-associated-definition",
        "mod owner; pub trait Provider { type Factory<X>; } pub struct Product; impl Product { pub fn ready() {} } pub struct Process; impl Provider for Process { type Factory<X> = Product; fn run() { Self::Factory::<u32>::ready(); } }",
        "pub fn own() { crate::Product::ready(); }",
        &call_owner("product-ready", "crate::Product::ready"),
    );

    assert_resolution(&repository, "src/lib.rs", 1);
}

fn repository(name: &str, body: &str, owner: &str, contract: &str) -> Repository {
    Repository::new(
        name,
        &format!(
            "mod owner; pub trait Factory {{ fn ready(); }} pub trait Provider {{ type Factory; }} pub struct Product; impl Product {{ pub fn ready() {{}} }} impl Factory for Product {{ fn ready() {{}} }} pub struct OtherFactory; impl OtherFactory {{ pub fn ready() {{}} }} pub struct OtherProduct; impl OtherProduct {{ pub fn ready() {{}} }} pub struct Process; {body}"
        ),
        owner,
        contract,
    )
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
