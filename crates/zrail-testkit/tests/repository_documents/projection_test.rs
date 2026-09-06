//! Typed `get/as_str` projections retain absent fields without granting absent required parents.

use super::{document, pass};
use crate::support::{coverage, violation};

#[test]
fn non_string_field_projection_preserves_types_literal_names_and_required_parents() {
    for format in ["toml", "json"] {
        let repository = document(
            format,
            "['dependency']",
            "op = 'field-not-string', field = 'version'",
        );
        let accepted = if format == "toml" {
            vec![
                "[dependency]",
                "dependency = false",
                "[dependency]\nversion = false",
                "[dependency]\nversion = 1",
                "[dependency]\nversion = 1.0",
                "[dependency]\nversion = []",
                "[dependency]\nVersion = '1'",
                "[unrelated]\nversion = '1'\n[dependency]",
            ]
        } else {
            vec![
                r#"{"dependency":{}}"#,
                r#"{"dependency":false}"#,
                r#"{"dependency":{"version":false}}"#,
                r#"{"dependency":{"version":1}}"#,
                r#"{"dependency":{"version":1.0}}"#,
                r#"{"dependency":{"version":[]}}"#,
                r#"{"dependency":{"Version":"1"}}"#,
                r#"{"unrelated":{"version":"1"},"dependency":{}}"#,
                r#"{"dependency":{"version":null}}"#,
                r#"{"dependency":null}"#,
            ]
        };
        for source in accepted {
            repository.write("metadata", source);
            pass(&repository);
        }
        for value in ["", "=0.1.0", "false"] {
            let source = if format == "toml" {
                format!("[dependency]\nversion = '{value}'")
            } else {
                format!("{{\"dependency\":{{\"version\":\"{value}\"}}}}")
            };
            repository.write("metadata", &source);
            violation(&repository, "metadata", "REP-FILE-007");
            let observed = coverage(&repository);
            let selected = observed.repository_files[0].entries[0]
                .document
                .as_ref()
                .expect("typed selection");
            assert_eq!(selected.selected_type.as_deref(), Some("object"));
            assert_eq!(selected.field_type.as_deref(), Some("string"));
        }
        repository.write(
            "metadata",
            if format == "toml" {
                "[unrelated]"
            } else {
                "{}"
            },
        );
        violation(&repository, "metadata", "REP-FILE-007");
        std::fs::remove_file(repository.0.join("metadata")).expect("remove required file");
        violation(&repository, "metadata", "REP-FILE-007");
    }
}

#[test]
fn wrong_intermediate_types_cannot_be_confused_with_a_missing_projected_field() {
    let repository = document(
        "toml",
        "['workspace', 'dependencies', 'sim']",
        "op = 'field-not-string', field = 'version'",
    );
    repository.write("metadata", "[workspace.dependencies.sim]\npath = 'sim'");
    pass(&repository);
    for source in [
        "workspace = false",
        "[workspace]\ndependencies = false",
        "[workspace.dependencies]",
    ] {
        repository.write("metadata", source);
        violation(&repository, "metadata", "REP-FILE-007");
    }
}
