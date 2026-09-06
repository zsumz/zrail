//! Each frozen authored key is independently deleted and aliased without changing unrelated inputs.

use zrail_core::{RepositoryDocumentAssertion, RepositoryFilePredicate, RepositoryFileRule};

use super::super::metadata::mutations::Mutation;

pub(super) fn cases(rule: &RepositoryFileRule, original: &[u8]) -> Vec<Mutation> {
    let RepositoryFilePredicate::Document(policy) = &rule.predicate else {
        panic!("expected document key predicate");
    };
    let subset = matches!(
        policy.assertion,
        RepositoryDocumentAssertion::KeysAllowed { .. }
    );
    let document = std::str::from_utf8(original)
        .expect("UTF-8 manifest")
        .parse::<toml::Value>()
        .expect("frozen TOML");
    let keys = document["dependencies"]
        .as_table()
        .expect("frozen dependencies")
        .keys()
        .cloned()
        .collect::<Vec<_>>();
    let mut cases = vec![
        raw("missing-input", None, Some("REP-FILE-007")),
        raw(
            "duplicate-key",
            Some([b"dependencies = false\n".as_slice(), original].concat()),
            Some("REP-FILE-006"),
        ),
        raw("invalid-utf8", Some(vec![0xff]), Some("REP-FILE-006")),
    ];
    let mut added = document.clone();
    dependencies(&mut added).insert(
        "unexpected-dependency".into(),
        toml::Value::String("1".into()),
    );
    cases.push(encoded("unexpected-key", &added, false));
    for key in &keys {
        let mut deleted = document.clone();
        dependencies(&mut deleted)
            .remove(key)
            .expect("existing frozen key");
        cases.push(encoded(format!("deleted-{key}"), &deleted, subset));
        // Explicit Cargo package spelling does not authorize an authored key alias.
        let mut alias = toml::map::Map::new();
        alias.insert("package".into(), toml::Value::String(key.clone()));
        alias.insert("version".into(), toml::Value::String("1".into()));
        dependencies(&mut deleted).insert(format!("alias-{key}"), toml::Value::Table(alias));
        cases.push(encoded(format!("aliased-{key}"), &deleted, false));
    }
    for (name, value) in [
        ("empty-table", toml::Value::Table(toml::map::Map::new())),
        ("present-boolean", toml::Value::Boolean(false)),
        ("present-integer", toml::Value::Integer(1)),
        ("present-string", toml::Value::String("dependencies".into())),
        ("present-array", toml::Value::Array(vec![])),
    ] {
        let mut changed = document.clone();
        changed
            .as_table_mut()
            .expect("root table")
            .insert("dependencies".into(), value);
        cases.push(encoded(name, &changed, subset));
    }
    let mut missing = document.clone();
    missing
        .as_table_mut()
        .expect("root table")
        .remove("dependencies");
    cases.push(encoded("missing-table", &missing, false));
    let mut moved = document.clone();
    let expected = moved
        .as_table_mut()
        .expect("root table")
        .insert(
            "dependencies".into(),
            toml::Value::Table(toml::map::Map::new()),
        )
        .expect("frozen selection");
    moved
        .as_table_mut()
        .expect("root table")
        .insert("dev-dependencies".into(), expected);
    cases.push(encoded("matching-keys-in-wrong-table", &moved, subset));
    let mut changed_values = document.clone();
    for (_, value) in dependencies(&mut changed_values).iter_mut() {
        *value = toml::Value::Boolean(false);
    }
    cases.push(encoded(
        "values-do-not-change-authored-names",
        &changed_values,
        true,
    ));
    let mut unrelated = document.clone();
    unrelated.as_table_mut().expect("root table").insert(
        "unrelated".into(),
        toml::Value::String("unexpected-dependency".into()),
    );
    cases.push(encoded("unrelated-field", &unrelated, true));
    let mut case_changed = document.clone();
    let value = dependencies(&mut case_changed)
        .remove(&keys[0])
        .expect("first key");
    dependencies(&mut case_changed).insert(keys[0].to_uppercase(), value);
    cases.push(encoded("case-changed-key", &case_changed, false));
    let mut oversized = document.clone();
    dependencies(&mut oversized).insert("x".repeat(16 * 1024), toml::Value::Boolean(false));
    cases.push(encoded("undisplayed-unexpected-key", &oversized, false));
    cases
}

fn dependencies(document: &mut toml::Value) -> &mut toml::map::Map<String, toml::Value> {
    document
        .get_mut("dependencies")
        .and_then(toml::Value::as_table_mut)
        .expect("mutable frozen table")
}

fn encoded(name: impl Into<String>, document: &toml::Value, accepted: bool) -> Mutation {
    raw(
        name,
        Some(
            toml::to_string(document)
                .expect("mutated TOML")
                .into_bytes(),
        ),
        if accepted { None } else { Some("REP-FILE-007") },
    )
}

fn raw(
    name: impl Into<String>,
    bytes: Option<Vec<u8>>,
    diagnostic: Option<&'static str>,
) -> Mutation {
    Mutation {
        name: name.into(),
        bytes,
        directory: false,
        diagnostic,
    }
}
