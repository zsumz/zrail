//! Exact raw lines preserve boundary and Unicode-trimming semantics without substring guesses.

use super::support::{configured, coverage, violation};
use zrail_core::ReportStatus;

#[test]
fn required_trimmed_lines_reject_comments_prefixes_and_suffixes() {
    let repository = configured(
        r#"
[[repository.files]]
name = "line"
include = ["attributes"]
reason = "Preserve exact trimmed Git attributes lines."
predicate = { kind = "literal", text = "*.rs text eol=lf", mode = "line-present", normalization = "trim" }
"#,
    );
    repository.write("attributes", "# Context\r\n\u{2003}*.rs text eol=lf \t\r\n");
    repository.lock();
    assert_eq!(repository.check().report.status, ReportStatus::Pass);
    assert_eq!(
        repository.explain("attributes").repository_files[0].claim,
        "raw-utf8-lines"
    );
    for source in [
        "# *.rs text eol=lf\n",
        "prefix *.rs text eol=lf\n",
        "*.rs text eol=lf suffix\n",
        "*.rs text\neol=lf\n",
        "",
    ] {
        repository.write("attributes", source);
        violation(&repository, "line", "REP-FILE-004");
    }
    std::fs::remove_file(repository.0.join("attributes")).expect("missing required input");
    violation(&repository, "line", "REP-FILE-004");
}

#[test]
fn forbidden_exact_name_lines_are_persistent_and_keep_untruncated_quantities() {
    let repository = configured(
        r#"
[[repository.files]]
name = "line"
include = ["names"]
reason = "Preserve the complete raw lock-name line scan."
predicate = { kind = "literal", text = 'name = "tokio"', mode = "line-absent", normalization = "trim" }
"#,
    );
    repository.lock();
    assert_eq!(repository.check().report.status, ReportStatus::Pass);
    for source in [
        "name = \"tokio-util-extra\"\n",
        "# name = \"tokio\"\n",
        "name = \"tokio\" # trailing\n",
        "name='tokio'\n",
        "name = \"Tokio\"\n",
    ] {
        repository.write("names", source);
        repository.lock();
        assert_eq!(repository.check().report.status, ReportStatus::Pass);
    }
    repository.write("names", &"\tname = \"tokio\"\r\n".repeat(20));
    violation(&repository, "line", "REP-FILE-004");
    let report = coverage(&repository);
    let entry = &report.repository_files[0].entries[0];
    assert_eq!(entry.literal_count, Some(20));
    assert_eq!(entry.literal_offsets.len(), 16);
    assert_eq!(entry.omitted_literal_offsets, 4);
}

#[test]
fn line_normalization_is_explicit_and_never_joins_different_lines() {
    for (normalization, positive, negative) in [
        ("none", "foo\r\n", " foo \n"),
        ("trim-start", "  foo\r\n", "  foo \n"),
        ("trim", "  foo \r\n", "foo suffix\n"),
        ("remove-whitespace", "f o o\r\n", "fo\no\n"),
    ] {
        let repository = configured(&format!(
            r#"
[[repository.files]]
name = "line"
include = ["lines"]
reason = "Explicit line boundaries and transformations."
predicate = {{ kind = "literal", text = "foo", mode = "line-present", normalization = "{normalization}" }}
"#
        ));
        repository.write("lines", positive);
        repository.lock();
        assert_eq!(repository.check().report.status, ReportStatus::Pass);
        repository.write("lines", negative);
        violation(&repository, "line", "REP-FILE-004");
    }
}
