//! Mutations preserve the original helper conjunction and isolate literal selected locations.

use zrail_core::{
    RepositoryDocumentAssertion as Assertion, RepositoryFilePredicate, RepositoryFileRule,
};

use super::super::metadata::mutations::{self, Mutation, field};

pub(super) fn cases(rule: &RepositoryFileRule, original: &[u8]) -> Vec<Mutation> {
    let RepositoryFilePredicate::Document(document) = &rule.predicate else {
        panic!("expected provenance document predicate");
    };
    match &document.assertion {
        Assertion::Absent => {
            let mut cases = vec![];
            for (name, value) in [
                ("present-false", toml::Value::Boolean(false)),
                ("present-string", toml::Value::String("disabled".into())),
                ("present-integer", toml::Value::Integer(0)),
                ("present-empty-array", toml::Value::Array(vec![])),
                (
                    "present-empty-table",
                    toml::Value::Table(toml::map::Map::new()),
                ),
            ] {
                cases.push(mutation(
                    name,
                    field(original, &document.path, Some(value)),
                    false,
                ));
            }
            cases.push(unrelated(original, &document.path));
            cases
        }
        Assertion::KeysExact { keys, .. } => {
            let mut cases = vec![
                mutation(
                    "missing-table",
                    field(original, &document.path, None),
                    false,
                ),
                mutation(
                    "non-table",
                    field(original, &document.path, Some(toml::Value::Boolean(false))),
                    false,
                ),
                mutation(
                    "empty-table",
                    field(
                        original,
                        &document.path,
                        Some(toml::Value::Table(toml::map::Map::new())),
                    ),
                    false,
                ),
            ];
            let table = keys
                .iter()
                .map(|key| (key.clone(), toml::Value::Boolean(true)))
                .collect::<toml::map::Map<_, _>>();
            for key in keys {
                let mut missing = table.clone();
                missing.remove(key).expect("required key");
                cases.push(mutation(
                    format!("missing-{key}"),
                    field(
                        original,
                        &document.path,
                        Some(toml::Value::Table(missing.clone())),
                    ),
                    false,
                ));
                missing.insert(key.to_uppercase(), toml::Value::Boolean(true));
                cases.push(mutation(
                    format!("same-count-case-changed-{key}"),
                    field(original, &document.path, Some(toml::Value::Table(missing))),
                    false,
                ));
            }
            for (name, key) in [
                ("unexpected-key", "path".to_owned()),
                ("undisplayed-key", "x".repeat(20_000)),
            ] {
                let mut extra = table.clone();
                extra.insert(key, toml::Value::String("elsewhere".into()));
                cases.push(mutation(
                    name,
                    field(original, &document.path, Some(toml::Value::Table(extra))),
                    false,
                ));
            }
            cases.push(unrelated(original, &document.path));
            cases
        }
        _ => mutations::cases(rule, original),
    }
}

fn unrelated(original: &[u8], selected: &[String]) -> Mutation {
    let value = selected
        .iter()
        .map(|key| (key.clone(), toml::Value::Boolean(true)))
        .collect();
    mutation(
        "matching-keys-in-unrelated-location",
        field(
            original,
            &["unrelated-metadata".into()],
            Some(toml::Value::Table(value)),
        ),
        true,
    )
}

fn mutation(name: impl Into<String>, bytes: Vec<u8>, accepted: bool) -> Mutation {
    Mutation {
        name: name.into(),
        bytes: Some(bytes),
        directory: false,
        diagnostic: if accepted { None } else { Some("REP-FILE-007") },
    }
}
