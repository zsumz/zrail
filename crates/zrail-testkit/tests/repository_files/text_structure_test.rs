//! Raw structure checks bind exact first occurrences, selected intervals, and complete line counts.

use super::fixture::Repository;
use super::support::{configured, coverage, violation};
use zrail_core::ReportStatus;
use zrail_rust::RepositoryTextStructureObservation as Observation;

fn repository(predicate: &str) -> Repository {
    configured(&format!(
        "[[repository.files]]\nname = 'shape'\ninclude = ['input']\nreason = 'Preserve exact raw structure.'\npredicate = {{ {predicate} }}"
    ))
}

#[test]
fn raw_order_requires_first_occurrences_and_reports_original_byte_positions() {
    let repository = repository("kind = 'literal-order', before = 'fetch', after = 'gate'");
    repository.write("input", "αfetch\ngate\n");
    repository.lock();
    assert_eq!(repository.check().report.status, ReportStatus::Pass);
    let explanation = repository.explain("input");
    assert_eq!(
        explanation.repository_files[0].claim,
        "raw-utf8-byte-interval"
    );
    assert_eq!(
        coverage(&repository).repository_files[0].entries[0].text_structure,
        Some(Observation::Order {
            before: Some(2),
            after: Some(8)
        })
    );
    for source in [
        "gate fetch",
        "gate fetch gate",
        "fetch",
        "gate",
        "",
        "FETCH gate",
    ] {
        repository.write("input", source);
        violation(&repository, "shape", "REP-FILE-008");
    }
    repository.write("input", "# fetch\n# gate\n");
    repository.lock();
    assert_eq!(
        repository.check().report.status,
        ReportStatus::Pass,
        "deliberately raw, not executable code order"
    );
    std::fs::remove_file(repository.0.join("input")).unwrap();
    violation(&repository, "shape", "REP-FILE-008");
}

#[test]
fn raw_between_uses_only_the_first_bounded_interval_and_keeps_complete_counts() {
    let repository =
        repository("kind = 'literal-between', start = 'START', end = 'END', contains = 'offline'");
    repository.write("input", "αSTART offline END offline");
    repository.lock();
    assert_eq!(repository.check().report.status, ReportStatus::Pass);
    let report = coverage(&repository);
    let entry = &report.repository_files[0].entries[0];
    assert_eq!(
        entry.text_structure,
        Some(Observation::Between {
            start: Some(2),
            end: Some(16)
        })
    );
    assert_eq!(entry.literal_count, Some(1));
    assert_eq!(entry.literal_offsets, [8]);
    for source in [
        "offline START END offline",
        "START END START offline END",
        "END START offline END",
        "START offline",
        "offline END",
        "START offENDline",
    ] {
        repository.write("input", source);
        violation(&repository, "shape", "REP-FILE-008");
    }
    repository.write(
        "input",
        &format!("START {}END offline", "offline ".repeat(25)),
    );
    repository.lock();
    assert_eq!(repository.check().report.status, ReportStatus::Pass);
    let report = coverage(&repository);
    let entry = &report.repository_files[0].entries[0];
    assert_eq!(entry.literal_count, Some(25));
    assert_eq!(entry.literal_offsets.len(), 16);
    assert_eq!(entry.omitted_literal_offsets, 9);
    repository.write(
        "input",
        &format!("START {}END changed outside", "offline ".repeat(25)),
    );
    assert!(
        repository
            .check()
            .report
            .findings
            .iter()
            .any(|finding| finding.id == "LOCK-028")
    );
}

#[test]
fn raw_interval_includes_its_start_and_excludes_its_end() {
    for (literal, accepted) in [("START", true), ("END", false)] {
        let repository = repository(&format!(
            "kind = 'literal-between', start = 'START', end = 'END', contains = '{literal}'"
        ));
        repository.write("input", "START END");
        repository.lock();
        if accepted {
            assert_eq!(repository.check().report.status, ReportStatus::Pass);
        } else {
            violation(&repository, "shape", "REP-FILE-008");
        }
    }
}

#[test]
fn prefixed_line_values_are_case_sensitive_complete_and_persistent_when_absent() {
    let repository = repository(
        "kind = 'line-values-allowed', prefix = 'image: ', values = ['one', 'two'], normalization = 'trim'",
    );
    repository.lock();
    assert_eq!(repository.check().report.status, ReportStatus::Pass);
    repository.write(
        "input",
        "\u{2003}image: one \r\nimage: two\nimage: one\n# image: wrong\n",
    );
    repository.lock();
    assert_eq!(repository.check().report.status, ReportStatus::Pass);
    assert!(matches!(
        coverage(&repository).repository_files[0].entries[0].text_structure,
        Some(Observation::LineValues {
            selected_count: 3,
            unauthorized_count: 0,
            ..
        })
    ));
    for source in [
        "image: One",
        "image: one # comment",
        "image: 'one'",
        "image: wrong\none",
    ] {
        repository.write("input", source);
        violation(&repository, "shape", "REP-FILE-008");
    }
    let source = format!(
        "image: {}\n{}",
        "x".repeat(20_000),
        "image: bad\n".repeat(20)
    );
    repository.write("input", &source);
    violation(&repository, "shape", "REP-FILE-008");
    let report = coverage(&repository);
    let Some(Observation::LineValues {
        selected_count,
        unauthorized_count,
        unauthorized_sample,
        unauthorized_omitted,
    }) = &report.repository_files[0].entries[0].text_structure
    else {
        panic!("line quantities")
    };
    assert_eq!(
        (*selected_count, *unauthorized_count, *unauthorized_omitted),
        (21, 21, 5)
    );
    assert_eq!(unauthorized_sample.len(), 16);
    assert_eq!(unauthorized_sample[0].line, 2);
}

#[test]
fn raw_structure_invalid_utf8_cannot_produce_partial_locks_or_coverage() {
    for predicate in [
        "kind = 'literal-order', before = 'a', after = 'b'",
        "kind = 'literal-between', start = 'a', end = 'b', contains = 'x'",
        "kind = 'line-values-allowed', prefix = 'image: ', values = []",
        "kind = 'line-prefixes-absent', prefixes = ['use tokio;']",
    ] {
        let repository = repository(predicate);
        std::fs::write(repository.0.join("input"), [0xff]).unwrap();
        for error in [
            zrail_rust::build_lock(&repository.0, "zrail.toml".as_ref()).unwrap_err(),
            zrail_rust::governed_surface_report(&repository.0, "zrail.toml".as_ref()).unwrap_err(),
        ] {
            assert!(error.to_string().contains("REP-FILE-006"), "{error}");
        }
    }
}
