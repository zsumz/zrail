//! Type aliases preserve prelude ADT identity without bypassing lexical shadows.

use super::fixture::{Repository, assert_no_owner, count, exact};

#[test]
fn prelude_option_alias_tuple_variant_reaches_base_owner() {
    let repository = fixture(
        "option-tuple",
        "type Maybe<T> = Option<T>; pub fn trespass() { let _ = Maybe::Some(1_u8); }",
    );
    exact(
        &repository.check(),
        "OWN-003",
        "option-construction",
        "src/lib.rs",
    );
}

#[test]
fn prelude_option_alias_unit_variant_reaches_base_owner() {
    let repository = fixture(
        "option-unit",
        "type Maybe<T> = Option<T>; pub fn trespass() { let _ = Maybe::<u8>::None; }",
    );
    exact(
        &repository.check(),
        "OWN-003",
        "option-construction",
        "src/lib.rs",
    );
}

#[test]
fn prelude_result_alias_variants_reach_base_owner() {
    let repository = fixture(
        "result-variants",
        "type Outcome<T, E> = Result<T, E>; pub fn trespass() { let _ = Outcome::<u8, u8>::Ok(1); let _ = Outcome::<u8, u8>::Err(2); }",
    );
    let report = repository.check();
    assert_eq!(
        count(&report, "OWN-003", "result-construction", "src/lib.rs"),
        2,
        "{}",
        report.human()
    );
}

#[test]
fn alias_chain_through_prelude_type_is_canonical() {
    let repository = fixture(
        "option-chain",
        "type Maybe<T> = Option<T>; type Second<T> = Maybe<T>; pub fn trespass() { let _ = Second::Some(1_u8); }",
    );
    exact(
        &repository.check(),
        "OWN-003",
        "option-construction",
        "src/lib.rs",
    );
}

#[test]
fn no_std_rejects_std_only_type_alias_target() {
    let repository = Repository::new(
        "no-std-alias",
        "2024",
        "#![no_std]\nextern crate std; mod owner; type Items<T> = Vec<T>; pub fn local() { let _ = Items::<u8>::new(); }",
    );
    let report = repository.check();
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.id == "RUST-INCLUDE-002"),
        "{}",
        report.human()
    );
    assert_eq!(count(&report, "OWN-003", "vec-new", "src/lib.rs"), 0);
}

#[test]
fn alias_generic_parameter_shadows_prelude_type() {
    let repository = fixture(
        "generic-option",
        "type Identity<Option> = Option; pub fn local() { let _ = Identity::<core::option::Option<u8>>::Some(1); }",
    );
    let report = repository.check();
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.id == "RUST-INCLUDE-002"),
        "{}",
        report.human()
    );
    assert_eq!(
        count(&report, "OWN-003", "option-construction", "src/lib.rs"),
        0
    );
}

#[test]
fn local_type_shadows_prelude_in_alias_target() {
    let repository = fixture(
        "local-option",
        "enum Option<T> { Some(T) } type Maybe<T> = Option<T>; pub fn local() { let _ = Maybe::Some(1_u8); }",
    );
    assert_no_owner(&repository.check(), "option-construction", "src/lib.rs");
}

fn fixture(name: &str, body: &str) -> Repository {
    Repository::new(name, "2024", &format!("mod owner; {body}"))
}
