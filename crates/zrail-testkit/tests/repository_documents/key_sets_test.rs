//! Exact authored dependency names cannot be substituted, hidden, or inferred from quantities.

use super::{document, pass};
use crate::support::{coverage, violation};

#[test]
fn exact_keys_require_the_same_names_and_allowed_keys_preserve_subset_semantics() {
    for format in ["toml", "json"] {
        for op in ["keys-exact", "keys-allowed"] {
            let repository = document(
                format,
                "['dependencies']",
                &format!("op = '{op}', keys = ['a', 'b']"),
            );
            let sources = if format == "toml" {
                [
                    "[dependencies]\nb = {}\na = '1'",
                    "[dependencies]\na = '1'",
                    "[dependencies]\na = '1'\nc = '1'",
                    "[unrelated]\na = '1'\nb = '1'",
                ]
            } else {
                [
                    r#"{"dependencies":{"b":{},"a":"1"}}"#,
                    r#"{"dependencies":{"a":"1"}}"#,
                    r#"{"dependencies":{"a":"1","c":"1"}}"#,
                    r#"{"unrelated":{"a":"1","b":"1"}}"#,
                ]
            };
            repository.write("metadata", sources[0]);
            pass(&repository);
            repository.write("metadata", sources[1]);
            if op == "keys-allowed" {
                pass(&repository);
            } else {
                violation(&repository, "metadata", "REP-FILE-007");
                let observed = coverage(&repository);
                let keys = observed.repository_files[0].entries[0]
                    .document
                    .as_ref()
                    .expect("document")
                    .keys
                    .as_ref()
                    .expect("keys");
                assert_eq!(keys.missing, ["b"]);
            }
            for source in &sources[2..] {
                repository.write("metadata", source);
                violation(&repository, "metadata", "REP-FILE-007");
            }
            std::fs::remove_file(repository.0.join("metadata")).expect("delete subject");
            violation(&repository, "metadata", "REP-FILE-007");
        }
    }
}

#[test]
fn non_table_empty_projection_is_explicit_and_never_grants_missing_subjects() {
    for op in ["keys-exact", "keys-allowed"] {
        for projection in [false, true] {
            let repository = document(
                "json",
                "['dependencies']",
                &format!("op = '{op}', keys = [], empty_on_non_table = {projection}"),
            );
            repository.write("metadata", r#"{"dependencies":{}}"#);
            pass(&repository);
            for value in ["null", "false", "1", "[]", "\"present string\""] {
                repository.write("metadata", &format!("{{\"dependencies\":{value}}}"));
                if projection {
                    pass(&repository);
                    let observed = coverage(&repository);
                    assert_eq!(
                        observed.repository_files[0].entries[0]
                            .document
                            .as_ref()
                            .expect("document")
                            .keys
                            .as_ref()
                            .expect("empty projected set")
                            .count,
                        0
                    );
                } else {
                    violation(&repository, "metadata", "REP-FILE-007");
                }
            }
            repository.write("metadata", "{}");
            violation(&repository, "metadata", "REP-FILE-007");
        }
    }
    let repository = document(
        "toml",
        "['dependencies']",
        "op = 'keys-exact', keys = ['required'], empty_on_non_table = true",
    );
    repository.write("metadata", "dependencies = false");
    violation(&repository, "metadata", "REP-FILE-007");
}

#[test]
fn key_samples_never_truncate_counts_comparison_or_bound_file_identity() {
    let repository = document(
        "json",
        "['dependencies']",
        "op = 'keys-exact', keys = ['required']",
    );
    repository.write("metadata", r#"{"dependencies":{"required":true}}"#);
    pass(&repository);
    let keys = (0..20)
        .map(|index| format!("\"unexpected-{index:02}\":true"))
        .collect::<Vec<_>>()
        .join(",");
    repository.write("metadata", &format!("{{\"dependencies\":{{{keys}}}}}"));
    violation(&repository, "metadata", "REP-FILE-007");
    let observed = coverage(&repository);
    let keys = observed.repository_files[0].entries[0]
        .document
        .as_ref()
        .expect("document")
        .keys
        .as_ref()
        .expect("key observations");
    assert_eq!((keys.count, keys.sample.len(), keys.omitted), (20, 16, 4));
    assert_eq!(
        (
            keys.unexpected_count,
            keys.unexpected_sample.len(),
            keys.unexpected_omitted
        ),
        (20, 16, 4)
    );
    assert_eq!(keys.missing, ["required"]);
    assert_eq!(
        observed.json().expect("coverage"),
        coverage(&repository).json().expect("repeated coverage")
    );
    let huge = "x".repeat(16 * 1024);
    repository.write(
        "metadata",
        &format!("{{\"dependencies\":{{\"{huge}\":true}}}}"),
    );
    violation(&repository, "metadata", "REP-FILE-007");
    let explained = repository.explain("metadata");
    let keys = explained.repository_files[0]
        .entry
        .as_ref()
        .expect("explained entry")
        .document
        .as_ref()
        .expect("document")
        .keys
        .as_ref()
        .expect("omitted long key");
    assert_eq!(
        (
            keys.count,
            keys.omitted,
            keys.unexpected_count,
            keys.unexpected_omitted
        ),
        (1, 1, 1, 1)
    );
    assert!(keys.sample.is_empty());
    assert!(keys.unexpected_sample.is_empty());
}
