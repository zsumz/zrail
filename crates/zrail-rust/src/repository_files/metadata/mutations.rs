//! Physical mutations isolate every authored field and preserve unrelated fixture inputs.

use zrail_core::{
    RepositoryDocumentAssertion as Assertion, RepositoryDocumentValue as Value,
    RepositoryFilePredicate, RepositoryFileRule,
};

pub(in super::super) struct Mutation {
    pub(in super::super) name: &'static str,
    pub(in super::super) bytes: Option<Vec<u8>>,
    pub(in super::super) directory: bool,
    pub(in super::super) diagnostic: Option<&'static str>,
}

pub(super) fn cases(rule: &RepositoryFileRule, original: &[u8]) -> Vec<Mutation> {
    let mutation = |name, bytes, diagnostic| Mutation {
        name,
        bytes,
        directory: false,
        diagnostic: Some(diagnostic),
    };
    match &rule.predicate {
        RepositoryFilePredicate::Document(document) if document.assertion == Assertion::Present => {
            vec![
                mutation("missing-input", None, "REP-FILE-007"),
                mutation("malformed-input", Some(b"[broken".to_vec()), "REP-FILE-006"),
                mutation(
                    "duplicate-key",
                    Some(b"key=1\nkey=2\n".to_vec()),
                    "REP-FILE-006",
                ),
                mutation("invalid-utf8", Some(vec![0xff]), "REP-FILE-006"),
            ]
        }
        RepositoryFilePredicate::Document(document) => {
            let mut values = match &document.assertion {
                Assertion::Equals {
                    value: Value::String(value),
                } => vec![
                    (
                        "wrong-value",
                        Some(toml::Value::String(format!("{value}-mutation"))),
                    ),
                    ("wrong-type", Some(toml::Value::Boolean(true))),
                ],
                Assertion::Equals {
                    value: Value::Boolean(value),
                } => vec![
                    ("wrong-value", Some(toml::Value::Boolean(!value))),
                    ("wrong-type", Some(toml::Value::String(value.to_string()))),
                ],
                Assertion::Equals {
                    value: Value::Strings(values),
                } => vec![
                    (
                        "wrong-value",
                        Some(toml::Value::Array(vec![toml::Value::String(
                            "unexpected".into(),
                        )])),
                    ),
                    (
                        "duplicate-entry",
                        Some(toml::Value::Array(
                            values
                                .iter()
                                .chain(values)
                                .cloned()
                                .map(toml::Value::String)
                                .collect(),
                        )),
                    ),
                    ("wrong-type", Some(toml::Value::Boolean(true))),
                ],
                Assertion::NonemptyString => vec![
                    ("empty", Some(toml::Value::String(String::new()))),
                    (
                        "unicode-whitespace",
                        Some(toml::Value::String("\u{2003}\t ".into())),
                    ),
                    ("wrong-type", Some(toml::Value::Integer(1))),
                ],
                other => panic!("unhandled frozen metadata assertion: {other:?}"),
            };
            values.push(("missing-field", None));
            values
                .into_iter()
                .map(|(name, value)| {
                    mutation(
                        name,
                        Some(field(original, &document.path, value)),
                        "REP-FILE-007",
                    )
                })
                .collect()
        }
        RepositoryFilePredicate::BytesEqual { other, utf8: true } => {
            let mut cases = vec![mutation("invalid-utf8", Some(vec![0xff]), "REP-FILE-005")];
            if rule.include[0] != *other {
                let mut bytes = original.to_vec();
                bytes.extend_from_slice(b"\nDifferent license text.\n");
                cases.push(mutation(
                    "different-valid-text",
                    Some(bytes),
                    "REP-FILE-005",
                ));
            }
            cases
        }
        RepositoryFilePredicate::Count { .. } => vec![
            mutation("missing-input", None, "REP-FILE-001"),
            Mutation {
                name: "wrong-entry-kind",
                bytes: None,
                directory: true,
                diagnostic: Some("REP-FILE-001"),
            },
        ],
        RepositoryFilePredicate::Literal(literal) => vec![mutation(
            "missing-marker",
            Some(
                std::str::from_utf8(original)
                    .expect("UTF-8 source")
                    .replace(&literal.text, "removed-marker")
                    .into_bytes(),
            ),
            "REP-FILE-004",
        )],
        other => panic!("unhandled frozen metadata predicate: {other:?}"),
    }
}

fn field(original: &[u8], path: &[String], replacement: Option<toml::Value>) -> Vec<u8> {
    let mut document = std::str::from_utf8(original)
        .expect("UTF-8 TOML")
        .parse::<toml::Value>()
        .expect("frozen manifest");
    let mut table = document.as_table_mut().expect("TOML root");
    let (last, parents) = path.split_last().expect("non-root field");
    for key in parents {
        table = table
            .get_mut(key)
            .and_then(toml::Value::as_table_mut)
            .expect("authored parent table");
    }
    if let Some(value) = replacement {
        table.insert(last.clone(), value);
    } else {
        assert!(table.remove(last).is_some());
    }
    toml::to_string(&document)
        .expect("mutated authored TOML")
        .into_bytes()
}
