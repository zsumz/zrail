//! Structural documents prove authored field contracts, never command execution.

#[path = "repository_documents/boundary_test.rs"]
mod boundary_test;
#[path = "strict_facades/fixture.rs"]
mod fixture;
#[path = "repository_files/support.rs"]
mod support;

use std::fs;

use fixture::Repository;
use support::{configured, coverage, violation};
use zrail_core::ReportStatus;

fn document(format: &str, path: &str, assertion: &str) -> Repository {
    configured(&format!(
        r#"
[[repository.files]]
name = "metadata"
include = ["metadata"]
reason = "Preserve the authored field assertion."
predicate = {{ kind = "document", format = "{format}", path = {path}, assertion = {{ {assertion} }} }}
"#
    ))
}

fn pass(repository: &Repository) {
    repository.lock();
    assert_eq!(repository.check().report.status, ReportStatus::Pass);
}

#[test]
fn exact_toml_metadata_cannot_be_satisfied_by_other_fields_or_comments() {
    let repository = document(
        "toml",
        "['workspace', 'package', 'version']",
        "op = 'equals', value = '0.1.0-rc.5'",
    );
    repository.write("metadata", "[workspace.package]\nversion = '0.1.0-rc.5'\n");
    pass(&repository);
    for source in [
        "[workspace.package]\nversion = '0.1.0-rc.6'\n# version = '0.1.0-rc.5'\n",
        "[unrelated]\nversion = '0.1.0-rc.5'\n",
        "['workspace.package']\nversion = '0.1.0-rc.5'\n",
        "[workspace.package]\nversion = 5\n",
    ] {
        repository.write("metadata", source);
        violation(&repository, "metadata", "REP-FILE-007");
    }
    fs::remove_file(repository.0.join("metadata")).expect("missing required file");
    violation(&repository, "metadata", "REP-FILE-007");
}

#[test]
fn workspace_inheritance_requires_authored_boolean_without_type_coercion() {
    let repository = document(
        "toml",
        "['package', 'version', 'workspace']",
        "op = 'equals', value = true",
    );
    repository.write("metadata", "[package]\nversion.workspace = true\n");
    pass(&repository);
    for source in [
        "[package]\nversion.workspace = 'true'\n",
        "[package]\nversion.workspace = false\n",
        "[package]\nversion = '0.1.0'\n",
        "[package]\nother.workspace = true\n",
    ] {
        repository.write("metadata", source);
        violation(&repository, "metadata", "REP-FILE-007");
    }
}

#[test]
fn exact_arrays_preserve_order_multiplicity_types_and_cardinality() {
    let repository = document("toml", "['publish']", "op = 'equals', value = ['a', 'b']");
    repository.write("metadata", "publish = ['a', 'b']");
    pass(&repository);
    for array in [
        "['b', 'a']",
        "['a', 'a']",
        "['a']",
        "['a', 'b', 'c']",
        "['a', 2]",
        "[]",
    ] {
        repository.write("metadata", &format!("publish = {array}"));
        violation(&repository, "metadata", "REP-FILE-007");
    }
}

#[test]
fn descriptions_require_nonempty_trimmed_strings_and_preserve_large_input_results() {
    let repository = document("toml", "['description']", "op = 'nonempty-string'");
    repository.write("metadata", "description = '  Useful metadata  '");
    pass(&repository);
    for value in ["''", "'\u{2003}\t '", "1"] {
        repository.write("metadata", &format!("description = {value}"));
        violation(&repository, "metadata", "REP-FILE-007");
    }
    repository.write(
        "metadata",
        &format!("description = '{}'", " ".repeat(17 * 1024)),
    );
    violation(&repository, "metadata", "REP-FILE-007");
    let observed = coverage(&repository);
    let selection = observed.repository_files[0].entries[0]
        .document
        .as_ref()
        .expect("selection");
    assert!(selection.value_omitted);
    assert!(selection.value.is_none());
    assert_eq!(selection.selected_type.as_deref(), Some("string"));
}

#[test]
fn forbidden_keys_accept_absence_but_do_not_confuse_null_or_wrong_shapes_with_missing() {
    let repository = document("json", "['package', 'private']", "op = 'absent'");
    pass(&repository);
    for source in ["{}", "{\"package\":{}}"] {
        repository.write("metadata", source);
        pass(&repository);
    }
    for source in ["{\"package\":{\"private\":null}}", "{\"package\":false}"] {
        repository.write("metadata", source);
        violation(&repository, "metadata", "REP-FILE-007");
    }
    let observed = coverage(&repository);
    assert!(
        observed.repository_files[0].entries[0]
            .document
            .as_ref()
            .expect("selection")
            .selection_error
            .is_some()
    );
}

#[test]
fn json_keys_are_literal_and_integer_expectations_do_not_accept_floats() {
    let repository = document(
        "json",
        "['on', 'value.with.dots']",
        "op = 'equals', value = 1",
    );
    repository.write("metadata", "{\"on\":{\"value.with.dots\":1}}");
    pass(&repository);
    for source in [
        "{\"on\":{\"value.with.dots\":1.0}}",
        "{\"ON\":{\"value.with.dots\":1}}",
        "{\"on\":{\"value\":{\"with\":{\"dots\":1}}}}",
        "{\"on\":{\"value.with.dots\":true}}",
    ] {
        repository.write("metadata", source);
        violation(&repository, "metadata", "REP-FILE-007");
    }
}

#[test]
fn complete_document_bytes_are_bound_even_when_the_field_still_passes() {
    let repository = document("json", "['required']", "op = 'present'");
    repository.write("metadata", "{\"required\":null}");
    pass(&repository);
    assert_eq!(
        coverage(&repository).json().expect("first coverage"),
        coverage(&repository).json().expect("repeated coverage")
    );
    let explained = repository.explain("metadata");
    assert_eq!(explained.repository_files[0].claim, "authored-document");
    assert!(explained.human().contains("repository:file:metadata"));
    assert!(
        coverage(&repository)
            .enabled_rails
            .contains(&"repository:file:metadata".into())
    );
    repository.write("metadata", "{\"required\":null,\"other\":true}");
    let report = repository.check().report;
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.id == "LOCK-028")
    );
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.id == "REP-FILE-007")
    );
}
