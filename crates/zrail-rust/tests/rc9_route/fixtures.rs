//! Closed route-source mutations bind all physical inputs and the intended native finding.

use std::{collections::BTreeMap, fs};

use serde_json::{Value, json};
use zrail_core::{RepositoryFileRule, sha256_hex};

use super::{
    compiler::Compiler,
    model::{FACADE, FORBIDDEN, Fixture, MARKER},
};

pub(super) fn inputs(fixture: &Fixture) -> BTreeMap<String, Value> {
    fixture
        .inputs
        .keys()
        .map(|path| {
            let value = match fs::read(fixture.root.join(path)) {
                Ok(bytes) => json!({"sha256": sha256_hex(&bytes), "bytes": bytes.len()}),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Value::Null,
                Err(error) => panic!("unexpected input read error: {error}"),
            };
            (path.clone(), value)
        })
        .collect()
}

pub(super) fn observe(
    fixture: &Fixture,
    policies: &[RepositoryFileRule],
    name: &str,
    expected: Option<(&str, &str)>,
) -> Value {
    let legacy_accepted = fixture.original_accepts();
    let (observations, findings, error) =
        match crate::repository_files::analyze(&fixture.root, policies) {
            Ok(analysis) => (
                analysis.policies,
                analysis
                    .findings
                    .into_iter()
                    .map(|finding| (finding.rule, finding.id))
                    .collect::<Vec<_>>(),
                None,
            ),
            Err(error) => (vec![], vec![], Some(error)),
        };
    let native_accepted = findings.is_empty() && error.is_none();
    assert_eq!(
        legacy_accepted,
        expected.is_none(),
        "{name}: original matcher"
    );
    assert_eq!(native_accepted, legacy_accepted, "{name}: native parity");
    if expected.is_some_and(|(_, id)| id == "REP-FILE-006") {
        assert!(findings.is_empty());
        assert!(
            error
                .as_ref()
                .expect("incomplete UTF-8")
                .starts_with("REP-FILE-006:")
        );
    } else {
        assert!(error.is_none(), "{name}: {error:?}");
        let mut expected = expected
            .into_iter()
            .map(|(rule, id)| (rule.into(), id.into()))
            .collect::<Vec<_>>();
        if name == format!("input-missing:{FACADE}") {
            expected.push((
                "repository:file:kd-route-owner-field".into(),
                "REP-FILE-004".into(),
            ));
        }
        assert_eq!(findings, expected, "{name}");
    }
    json!({"case": name, "inputs": inputs(fixture), "legacy_accepted": legacy_accepted,
           "native_accepted": native_accepted, "observations": observations,
           "findings": findings, "error": error})
}

pub(super) fn qualify(
    fixture: &Fixture,
    policies: &[RepositoryFileRule],
    compiler: &Compiler,
) -> Vec<Value> {
    let mut rows = vec![observe(fixture, policies, "valid", None)];
    rows[0]["compiler"] = compiler.compile(None);
    for (path, original) in &fixture.inputs {
        if path == FACADE {
            continue;
        }
        for token in FORBIDDEN {
            let policy = format!("repository:file:kd-route-forbid-{token}");
            for (context, addition) in [
                ("comment", format!("\n// {token}\n")),
                ("string", format!("\nconst TEXT: &str = \"{token}\";\n")),
                ("rename", format!("\nuse crate::{token} as Renamed;\n")),
                ("lowercase", format!("\n// {}\n", token.to_lowercase())),
            ] {
                fixture.write(path, format!("{original}{addition}"));
                let expected =
                    (context != "lowercase").then_some((policy.as_str(), "REP-FILE-004"));
                rows.push(observe(
                    fixture,
                    policies,
                    &format!("{context}:{path}:{token}"),
                    expected,
                ));
            }
        }
        fixture.write(path, original);
    }
    fixture.write("unrelated.rs", FORBIDDEN.join(" "));
    rows.push(observe(fixture, policies, "unrelated-tokens", None));
    let original = &fixture.inputs[FACADE];
    let removed = original.replace(MARKER, "connections: Other<T>");
    for (name, source) in [
        ("owner-missing", removed.clone()),
        ("owner-duplicate", format!("{original}\n// {MARKER}\n")),
        (
            "owner-spelling",
            original.replace(MARKER, "connections : DirectSetOwner<T>"),
        ),
        ("owner-wrong-file", removed.clone()),
        ("owner-comment", format!("{removed}\n// {MARKER}\n")),
    ] {
        fixture.write("unrelated.rs", MARKER);
        fixture.write(FACADE, source);
        rows.push(observe(
            fixture,
            policies,
            name,
            (name != "owner-comment")
                .then_some(("repository:file:kd-route-owner-field", "REP-FILE-004")),
        ));
    }
    fixture.write(FACADE, original);
    fs::remove_file(fixture.root.join("unrelated.rs")).expect("remove owned decoy");
    for (path, original) in &fixture.inputs {
        for invalid in [false, true] {
            let (case, diagnostic) = if invalid {
                fixture.write(path, [0xff]);
                ("input-utf8", "REP-FILE-006")
            } else {
                fs::remove_file(fixture.root.join(path)).expect("delete owned input");
                ("input-missing", "REP-FILE-002")
            };
            let mut row = observe(
                fixture,
                policies,
                &format!("{case}:{path}"),
                Some(("repository:file:kd-route-inputs", diagnostic)),
            );
            row["compiler"] = compiler.compile(Some((path, invalid)));
            rows.push(row);
            fixture.write(path, original);
        }
    }
    assert_eq!(rows.len(), 189);
    assert_eq!(
        rows.iter()
            .filter(|row| row["native_accepted"] == true)
            .count(),
        43
    );
    rows
}
