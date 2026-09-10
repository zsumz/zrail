//! Prefix prohibitions retain physical line quantities, bounded samples, and input authority.

use super::support::{configured, coverage, violation};
use zrail_core::ReportStatus;
use zrail_rust::RepositoryTextStructureObservation as Observation;

#[test]
fn prefix_sets_count_each_line_once_and_expose_every_omitted_violation() {
    let repository = configured(
        "[[repository.files]]\nname = 'prefix'\ninclude = ['input']\nreason = 'Reviewed raw boundary.'\npredicate = { kind = 'line-prefixes-absent', prefixes = ['use tokio;', 'use tokio', 'extern crate tokio;'], normalization = 'trim-start' }",
    );
    repository.lock();
    assert_eq!(repository.check().report.status, ReportStatus::Pass);
    repository.write(
        "input",
        "Use tokio;\n// use tokio;\nlet s = \"use tokio;\";\n",
    );
    repository.lock();
    assert_eq!(repository.check().report.status, ReportStatus::Pass);
    let explanation = repository.explain("input");
    assert_eq!(
        explanation.repository_files[0].claim,
        "raw-utf8-line-prefixes"
    );
    repository.write(
        "input",
        &format!("safe\n{}", "\u{2003}use tokio;\r\n".repeat(21)),
    );
    violation(&repository, "prefix", "REP-FILE-008");
    let report = coverage(&repository);
    let observed = &report.repository_files[0];
    assert_eq!(observed.policy_id, "repository:file:prefix");
    assert_eq!(observed.analysis, zrail_core::AnalysisQuality::Exact);
    assert_eq!(
        observed.entries[0].text_structure,
        Some(Observation::LinePrefixes {
            forbidden_count: 21,
            forbidden_lines: (2..18).collect(),
            forbidden_omitted: 5,
        })
    );
    assert_eq!(
        report,
        coverage(&repository),
        "repeat analysis is deterministic"
    );
}

#[test]
fn prefix_absence_remains_live_and_binds_bytes_outside_selected_lines() {
    let repository = configured(
        "[[repository.files]]\nname = 'prefix'\ninclude = ['input']\nreason = 'Reviewed raw boundary.'\npredicate = { kind = 'line-prefixes-absent', prefixes = ['use tokio;'] }",
    );
    repository.write("input", "safe\n");
    repository.lock();
    repository.write("input", "changed safe text\n");
    assert!(
        repository
            .check()
            .report
            .findings
            .iter()
            .any(|f| f.id == "LOCK-028")
    );
    repository.write("input", "use tokio;");
    violation(&repository, "prefix", "REP-FILE-008");
    std::fs::remove_file(repository.0.join("input")).unwrap();
    repository.lock();
    assert_eq!(repository.check().report.status, ReportStatus::Pass);
    repository.write("input", "use tokio;\n");
    violation(&repository, "prefix", "REP-FILE-008");
}
