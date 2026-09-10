//! Raw structures reject unknown execution modes and bound every selected literal/value.

use crate::contract::validate_fixture_test::minimal_contract;

#[test]
fn raw_structure_schemas_are_closed_and_bound_expected_content() {
    for predicate in [
        "kind = 'literal-order', before = 'fetch', after = 'gate'",
        "kind = 'literal-between', start = 'gate', end = 'package', contains = 'offline'",
        "kind = 'line-values-allowed', prefix = 'image: ', values = ['pinned'], normalization = 'trim'",
        "kind = 'line-values-allowed', prefix = 'image: ', values = []",
        "kind = 'line-prefixes-absent', prefixes = ['use tokio;', 'extern crate tokio '], normalization = 'trim-start'",
    ] {
        assert!(errors(predicate).is_empty());
        assert!(
            toml::from_str::<crate::RepositoryFileRule>(&rule(&format!(
                "{predicate}, plugin = 'shell'"
            )))
            .is_err()
        );
    }
    for predicate in [
        "kind = 'literal-order', before = '', after = 'gate'".into(),
        format!("kind = 'literal-between', start = 'a', end = 'b', contains = '{}'", "x".repeat(16385)),
        "kind = 'line-values-allowed', prefix = '', values = []".into(),
        "kind = 'line-values-allowed', prefix = 'image: ', values = ['a', 'a']".into(),
        "kind = 'line-values-allowed', prefix = 'image: ', values = [], normalization = 'remove-whitespace'".into(),
        "kind = 'line-values-allowed', prefix = \"image:\\n\", values = []".into(),
        format!("kind = 'line-values-allowed', prefix = 'image: ', values = ['{}']", "x".repeat(16385)),
    ] {
        assert!(!errors(&predicate).is_empty(), "{predicate}");
    }
}

#[test]
fn forbidden_line_prefixes_have_bounded_canonical_literal_inputs() {
    for predicate in [
        "prefixes = []".into(),
        "prefixes = ['']".into(),
        "prefixes = ['a', 'a']".into(),
        "prefixes = [\"a\\n\"]".into(),
        "prefixes = [\"a\\r\"]".into(),
        "prefixes = [' a'], normalization = 'trim-start'".into(),
        "prefixes = ['a'], normalization = 'remove-whitespace'".into(),
        format!("prefixes = ['{}']", "x".repeat(16385)),
        format!(
            "prefixes = [{}]",
            (0..65)
                .map(|i| format!("'p{i}'"))
                .collect::<Vec<_>>()
                .join(",")
        ),
    ] {
        assert!(!errors(&format!("kind = 'line-prefixes-absent', {predicate}")).is_empty());
    }
    for predicate in [
        "prefixes = ['a', 'ab']",
        "prefixes = [' a'], normalization = 'none'",
        "prefixes = ['a '], normalization = 'trim'",
    ] {
        assert!(errors(&format!("kind = 'line-prefixes-absent', {predicate}")).is_empty());
    }
}

fn errors(predicate: &str) -> Vec<String> {
    let mut contract = minimal_contract();
    contract.repository.files = vec![toml::from_str(&rule(predicate)).expect("typed rule")];
    let mut errors = super::ValidationErrors::new();
    super::super::validate(&contract, &mut errors);
    errors.finish()
}

fn rule(predicate: &str) -> String {
    format!(
        "name = 'raw'\ninclude = ['ci.yml']\nreason = 'Reviewed raw structure.'\npredicate = {{ {predicate} }}"
    )
}
