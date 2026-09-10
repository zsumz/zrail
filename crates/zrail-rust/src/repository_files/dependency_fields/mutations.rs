//! Typed projections and ordered feature mutations extend the shared authored-value fixtures.

use zrail_core::{
    RepositoryDocumentAssertion as Assertion, RepositoryDocumentValue as Value,
    RepositoryFilePredicate, RepositoryFileRule,
};

use super::super::metadata::mutations::{self, Mutation, field};

pub(super) fn cases(rule: &RepositoryFileRule, original: &[u8]) -> Vec<Mutation> {
    let RepositoryFilePredicate::Document(document) = &rule.predicate else {
        panic!("expected authored field");
    };
    if let Assertion::FieldNotString { field: key } = &document.assertion {
        let mut selected = document.path.clone();
        selected.push(key.clone());
        let mut cases = Vec::new();
        for (name, value, accepted) in [
            ("boolean-version", toml::Value::Boolean(false), true),
            ("integer-version", toml::Value::Integer(1), true),
            ("array-version", toml::Value::Array(vec![]), true),
            (
                "empty-string-version",
                toml::Value::String(String::new()),
                false,
            ),
            (
                "string-version",
                toml::Value::String("=0.1.0".into()),
                false,
            ),
        ] {
            cases.push(mutation(
                name,
                Some(field(original, &selected, Some(value))),
                accepted,
            ));
        }
        cases.push(mutation(
            "present-non-table-parent",
            Some(field(
                original,
                &document.path,
                Some(toml::Value::Boolean(false)),
            )),
            true,
        ));
        cases.push(mutation(
            "missing-parent",
            Some(field(original, &document.path, None)),
            false,
        ));
        cases.push(mutation("missing-input", None, false));
        return cases;
    }
    let mut cases = mutations::cases(rule, original);
    if let Assertion::Equals {
        value: Value::Strings(values),
    } = &document.assertion
    {
        cases.push(mutation(
            "empty-array",
            Some(field(
                original,
                &document.path,
                Some(toml::Value::Array(vec![])),
            )),
            false,
        ));
        if values.len() > 1 {
            let mut reordered = values.clone();
            reordered.reverse();
            let mut duplicate = values.clone();
            duplicate[1] = duplicate[0].clone();
            for (name, values) in [
                ("reordered-array", reordered),
                ("missing-array-item", values[1..].to_vec()),
                ("same-count-duplicate", duplicate),
            ] {
                let value =
                    toml::Value::Array(values.into_iter().map(toml::Value::String).collect());
                cases.push(mutation(
                    name,
                    Some(field(original, &document.path, Some(value))),
                    false,
                ));
            }
        }
    }
    cases
}

fn mutation(name: &str, bytes: Option<Vec<u8>>, accepted: bool) -> Mutation {
    Mutation {
        name: name.into(),
        bytes,
        directory: false,
        diagnostic: if accepted { None } else { Some("REP-FILE-007") },
    }
}
