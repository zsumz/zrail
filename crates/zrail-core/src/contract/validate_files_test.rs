//! Closed file schemas reject ambiguity before accepting any repository authority.

use super::*;
use crate::contract::validate_fixture_test::minimal_contract;

const RULE: &str = r#"
name = "leading-contract"
include = ["crates/**/*.rs"]
reason = "Preserve the reviewed leading literal."
predicate = { kind = "literal", mode = "starts-with", text = "//!", normalization = "trim-start" }
"#;

#[test]
fn file_predicates_reject_unknown_fields_and_unsupported_modes() {
    let _: crate::RepositoryFileRule = toml::from_str(RULE).expect("typed literal predicate");
    for invalid in [
        format!("{RULE}\nplugin = 'checker.py'"),
        RULE.replace("kind = \"literal\"", "kind = 'rust-expression'"),
        RULE.replace("mode = \"starts-with\"", "mode = 'regex'"),
        RULE.replace(
            "normalization = \"trim-start\"",
            "normalization = 'parse-rust'",
        ),
        RULE.replace("text = \"//!\"", "text = '//!', unknown = true"),
        RULE.replace("include =", "include = ['src/**']\ninclude ="),
    ] {
        assert!(
            toml::from_str::<crate::RepositoryFileRule>(&invalid).is_err(),
            "{invalid}"
        );
    }
}

#[test]
fn file_assertions_validate_selection_cardinality_identity_and_literals() {
    assert!(errors(RULE).is_empty());
    for (invalid, expected) in [
        (
            RULE.replace("name = \"leading-contract\"", "name = ''"),
            "unique nonempty",
        ),
        (
            RULE.replace("include = [\"crates/**/*.rs\"]", "include = []"),
            "1..64 selectors",
        ),
        (
            RULE.replace("crates/**/*.rs", "crates//*.rs"),
            "not canonical",
        ),
        (
            RULE.replace("crates/**/*.rs", "../src/*.rs"),
            "not canonical",
        ),
        (RULE.replace("text = \"//!\"", "text = ''"), "UTF-8 bytes"),
        (
            RULE.replace("text = \"//!\"", "text = '//!', count = 1"),
            "required only",
        ),
        (RULE.replace("starts-with", "exact-count"), "required only"),
        (
            RULE.replace("trim-start", "remove-whitespace")
                .replace("//!", "// !"),
            "must not themselves",
        ),
        (
            format!("{RULE}\nentry = 'directory'"),
            "select file entries",
        ),
    ] {
        assert!(
            errors(&invalid).contains(expected),
            "{invalid}: {}",
            errors(&invalid)
        );
    }
}

#[test]
fn zero_count_and_empty_exact_inventory_are_live_prohibitions() {
    for predicate in [
        "{ kind = 'count', minimum = 0, maximum = 0 }",
        "{ kind = 'exact-paths', paths = [] }",
        "{ kind = 'literal', text = 'retired', mode = 'absent' }",
        "{ kind = 'literal', text = 'retired', mode = 'exact-count', count = 0 }",
    ] {
        assert!(errors(&with_predicate(predicate)).is_empty());
    }
    for predicate in [
        "{ kind = 'count', minimum = 0 }",
        "{ kind = 'count', minimum = 5, maximum = 4 }",
        "{ kind = 'exact-paths', paths = ['src/lib.rs'] }",
        "{ kind = 'forbidden-names', names = [], part = 'component-stem' }",
        "{ kind = 'bytes-equal', other = 'src//LICENSE' }",
    ] {
        assert!(
            !errors(&with_predicate(predicate)).is_empty(),
            "{predicate}"
        );
    }
}

#[test]
fn old_repository_contract_serialization_omits_the_new_empty_family() {
    let contract = minimal_contract();
    assert!(contract.repository.files.is_empty());
    assert!(
        !toml::to_string(&contract.repository)
            .expect("serialize repository")
            .contains("files")
    );
}

fn with_predicate(predicate: &str) -> String {
    format!(
        "{}predicate = {predicate}\n",
        RULE.split("predicate =").next().expect("rule prefix")
    )
}

fn errors(source: &str) -> String {
    let mut contract = minimal_contract();
    contract.repository.files = vec![toml::from_str(source).expect("parse typed fixture")];
    let mut errors = ValidationErrors::new();
    validate(&contract, &mut errors);
    errors.finish().join("\n")
}
