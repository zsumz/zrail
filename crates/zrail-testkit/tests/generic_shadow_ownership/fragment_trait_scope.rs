//! Fragment-local associated types belong to the trait being implemented.

use super::fixture::{Repository, call_owner, finding_count};

const SOURCE: &str = r#"mod owner;

pub trait Provider {
    type Factory;
    fn run();
}

pub trait Other {
    type Factory;
}

pub struct Product;
impl Product {
    pub fn ready() {}
}

pub struct OtherProduct;
impl OtherProduct {
    pub fn ready() {}
}

pub struct Process;

impl Other for Process {
    type Factory = OtherProduct;
}

impl Provider for Process
where
    Self: Other<Factory = OtherProduct>,
{
    include!("impl_items.inc");
}
"#;

const FRAGMENT: &str = "type Factory = Product; fn run() { <Self as Other>::Factory::ready(); }";

#[test]
fn fragment_associated_definition_does_not_cross_an_explicit_trait() {
    let product = repository(
        "fragment-current-trait-not-other",
        "pub fn own() { crate::Product::ready(); }",
        &call_owner("product-ready", "crate::Product::ready"),
    );

    assert_resolution(&product, 0);
}

#[test]
fn filtered_fragment_candidate_is_removed_without_a_matching_bound() {
    let source = SOURCE.replace(
        "impl Provider for Process\nwhere\n    Self: Other<Factory = OtherProduct>,\n{",
        "impl Provider for Process {",
    );
    let product = repository_with_source(
        "fragment-current-trait-no-other-bound",
        &source,
        "pub fn own() { crate::Product::ready(); }",
        &call_owner("product-ready", "crate::Product::ready"),
    );

    assert_resolution(&product, 0);
}

#[test]
fn inherited_explicit_trait_equality_remains_additive() {
    let other = repository(
        "fragment-inherited-other-equality",
        "pub fn own() { crate::OtherProduct::ready(); }",
        &call_owner("other-ready", "crate::OtherProduct::ready"),
    );

    assert_resolution(&other, 1);
}

fn repository(name: &str, owner: &str, contract: &str) -> Repository {
    repository_with_source(name, SOURCE, owner, contract)
}

fn repository_with_source(name: &str, source: &str, owner: &str, contract: &str) -> Repository {
    let repository = Repository::new(name, source, owner, contract);
    repository.write("src/impl_items.inc", FRAGMENT);
    repository
}

fn assert_resolution(repository: &Repository, expected: usize) {
    let report = repository.check();
    assert_eq!(
        finding_count(
            &report,
            "RUST-CALL-001",
            "rust.source.call-resolution",
            "src/impl_items.inc",
        ),
        expected,
        "{}",
        report.human(),
    );
}
