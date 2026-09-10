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
            RULE.replace("crates/**/*.rs", "crates/./*.rs"),
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

#[test]
fn impossible_line_literals_and_line_count_fields_are_rejected() {
    for predicate in [
        "{ kind = 'literal', text = '\n', mode = 'line-absent' }",
        "{ kind = 'literal', text = ' padded', mode = 'line-absent', normalization = 'trim-start' }",
        "{ kind = 'literal', text = 'padded ', mode = 'line-present', normalization = 'trim' }",
        "{ kind = 'literal', text = 'line', mode = 'line-present', count = 1 }",
    ] {
        // Literal TOML strings may contain an actual newline only in triple quotes.
        let predicate = predicate.replace("text = '\n'", "text = \"\\n\"");
        assert!(!errors(&with_predicate(&predicate)).is_empty());
    }
}

#[test]
fn document_policy_is_typed_bounded_and_has_no_expression_or_parser_fallback() {
    let valid = with_predicate(
        "{ kind = 'document', format = 'toml', path = ['package', 'publish'], assertion = { op = 'equals', value = ['crates-io'] } }",
    );
    assert!(errors(&valid).is_empty());
    for invalid in [
        valid.replace("format = 'toml'", "format = 'yaml'"),
        valid.replace("path = ['package', 'publish']", "path = '$..publish'"),
        valid.replace("value = ['crates-io']", "value = [true]"),
        valid.replace("value = ['crates-io']", "value = 1.0"),
        valid.replace("value = ['crates-io']", "value = { arbitrary = true }"),
        valid.replace("op = 'equals'", "op = 'expression'"),
        valid.replace("op = 'equals'", "op = 'equals', unknown = true"),
    ] {
        assert!(
            toml::from_str::<crate::RepositoryFileRule>(&invalid).is_err(),
            "{invalid}"
        );
    }
    let long_path = format!("path = [{}'last']", "'key',".repeat(32));
    assert!(
        errors(&valid.replace("path = ['package', 'publish']", &long_path))
            .contains("32 literal keys")
    );
    let large_value = format!("value = '{}'", "x".repeat(16 * 1024 + 1));
    assert!(errors(&valid.replace("value = ['crates-io']", &large_value)).contains("16384"));
}

#[test]
fn document_key_sets_reject_duplicates_and_bound_all_expected_names() {
    for op in ["keys-exact", "keys-allowed"] {
        let rule = with_predicate(&format!(
            "{{ kind = 'document', format = 'toml', path = ['dependencies'], assertion = {{ op = '{op}', keys = [] }} }}"
        ));
        assert!(errors(&rule).is_empty());
        assert!(errors(&rule.replace("keys = []", "keys = ['a', 'a']")).contains("duplicate"));
        assert!(
            errors(&rule.replace("keys = []", &format!("keys = ['{}']", "x".repeat(16385))))
                .contains("16384")
        );
        let keys = (0..1025)
            .map(|key| format!("'{key}'"))
            .collect::<Vec<_>>()
            .join(",");
        assert!(errors(&rule.replace("keys = []", &format!("keys = [{keys}]"))).contains("1024"));
        let parsed: crate::RepositoryFileRule = toml::from_str(&rule).expect("default strict keys");
        assert!(
            !toml::to_string(&parsed)
                .expect("default omitted")
                .contains("empty_on_non_table")
        );
        for invalid in [
            "keys = [1]",
            "keys = [], unknown = true",
            "keys = [], empty_on_non_table = 'true'",
        ] {
            assert!(
                toml::from_str::<crate::RepositoryFileRule>(&rule.replace("keys = []", invalid))
                    .is_err()
            );
        }
    }
}

#[test]
fn field_projections_require_one_literal_bounded_key_without_optional_parent_permissions() {
    let rule = with_predicate(
        "{ kind = 'document', format = 'toml', path = ['dependency'], assertion = { op = 'field-not-string', field = 'version' } }",
    );
    assert!(errors(&rule).is_empty());
    assert!(
        errors(&rule.replace(
            "field = 'version'",
            &format!("field = '{}'", "x".repeat(1025))
        ))
        .contains("1024")
    );
    for invalid in [
        "field = ['version']",
        "field = 1",
        "field = 'version', optional_subject = true",
        "unknown = 'version'",
    ] {
        assert!(
            toml::from_str::<crate::RepositoryFileRule>(
                &rule.replace("field = 'version'", invalid)
            )
            .is_err()
        );
    }
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

#[test]
fn non_directory_entries_support_path_predicates_but_never_content_reads() {
    for predicate in [
        "{ kind = 'count', minimum = 0, maximum = 0 }",
        "{ kind = 'exact-paths', paths = [] }",
        "{ kind = 'forbidden-names', names = ['old'], part = 'file-stem' }",
    ] {
        let rule = format!("{}entry = 'non-directory'\n", with_predicate(predicate));
        assert!(errors(&rule).is_empty());
        let parsed: crate::RepositoryFileRule = toml::from_str(&rule).expect("new entry mode");
        assert_eq!(parsed.entry, crate::RepositoryEntryMode::NonDirectory);
        let encoded = toml::to_string(&parsed).expect("serialize entry mode");
        assert!(encoded.contains("non-directory"));
        assert_eq!(
            toml::from_str::<crate::RepositoryFileRule>(&encoded).expect("round trip"),
            parsed
        );
    }
    for predicate in [
        "{ kind = 'literal', text = 'old', mode = 'absent' }",
        "{ kind = 'bytes-equal', other = 'LICENSE' }",
        "{ kind = 'document', format = 'toml', path = ['package'], assertion = { op = 'absent' } }",
    ] {
        assert!(
            errors(&format!(
                "{}entry = 'non-directory'\n",
                with_predicate(predicate)
            ))
            .contains("select file entries")
        );
    }
    assert!(
        toml::from_str::<crate::RepositoryFileRule>(&format!("{RULE}\nentry = 'non-regular'"))
            .is_err()
    );
}
