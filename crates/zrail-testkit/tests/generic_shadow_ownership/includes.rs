//! Expression includes retain the namespace of inherited generic parameters.

use super::fixture::{
    Repository, assert_complete, assert_no_owner, call_owner, construction_owner, exact_owner_count,
};

#[test]
fn included_type_generic_does_not_shadow_bare_drop() {
    let repository = Repository::new(
        "included-drop",
        "mod owner; pub fn wrap<drop>(value: u8) { include!(\"expr.rs\"); }",
        "pub fn own() { core::mem::drop(0_u8); }",
        &call_owner("drop-call", "core::mem::drop"),
    );
    repository.write("src/expr.rs", "{ drop(value); }");
    let report = repository.check();
    assert_complete(&report);
    assert_eq!(exact_owner_count(&report, "drop-call", "src/expr.rs"), 1);
}

#[test]
fn included_type_generic_does_not_shadow_some_or_none() {
    let repository = option_fixture(
        "included-options",
        "mod owner; pub fn wrap<Some, None>(value: u8) -> (Option<u8>, Option<u8>) { include!(\"expr.rs\") }",
    );
    repository.write("src/expr.rs", "(Some(value), None)");
    let report = repository.check();
    assert_complete(&report);
    assert_eq!(
        exact_owner_count(&report, "option-construction", "src/expr.rs"),
        2,
        "{}",
        report.human()
    );
}

#[test]
fn included_type_generic_shadows_qualified_type_root() {
    let repository = Repository::new(
        "included-generic-root",
        "mod owner; pub enum Choice { Ready(u64) } pub fn real() -> Choice { Choice::Ready(1) } pub trait Factory { #[allow(non_snake_case)] fn Ready(value: u64) -> Self; } pub fn make<Choice: Factory>() -> Choice { include!(\"expr.rs\") }",
        "pub fn own() -> crate::Choice { crate::Choice::Ready(1) }",
        &construction_owner("choice-construction", "crate::Choice"),
    );
    repository.write("src/expr.rs", "Choice::Ready(44)");
    let report = repository.check();
    assert_eq!(
        exact_owner_count(&report, "choice-construction", "src/lib.rs"),
        1,
        "{}",
        report.human()
    );
    assert_no_owner(&report, "choice-construction", "src/expr.rs");
}

#[test]
fn nested_include_preserves_generic_namespace() {
    let repository = option_fixture(
        "nested-options",
        "mod owner; pub fn wrap<Some>(value: u8) -> Option<u8> { include!(\"outer.rs\") }",
    );
    repository.write("src/outer.rs", "{ include!(\"inner.rs\") }");
    repository.write("src/inner.rs", "Some(value)");
    let report = repository.check();
    assert_complete(&report);
    assert_eq!(
        exact_owner_count(&report, "option-construction", "src/inner.rs"),
        1,
        "{}",
        report.human()
    );
}

#[test]
fn direct_and_included_generic_rules_are_identical() {
    let repository = option_fixture(
        "direct-included-options",
        "mod owner; pub fn direct<Some>(value: u8) -> Option<u8> { Some(value) } pub fn included<Some>(value: u8) -> Option<u8> { include!(\"expr.rs\") }",
    );
    repository.write("src/expr.rs", "Some(value)");
    let report = repository.check();
    assert_complete(&report);
    assert_eq!(
        exact_owner_count(&report, "option-construction", "src/lib.rs"),
        1,
        "{}",
        report.human()
    );
    assert_eq!(
        exact_owner_count(&report, "option-construction", "src/expr.rs"),
        1,
        "{}",
        report.human()
    );
}

fn option_fixture(name: &str, source: &str) -> Repository {
    Repository::new(
        name,
        source,
        "pub fn own(value: u8) -> (Option<u8>, Option<u8>) { (Some(value), None) }",
        &construction_owner("option-construction", "core::option::Option"),
    )
}
