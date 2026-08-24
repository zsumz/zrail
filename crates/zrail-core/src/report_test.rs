//! Report status and deterministic JSON.

use crate::diagnostic::{DiagnosticLimit, Finding, Severity};

use super::{Report, ReportStatus};

#[test]
fn errors_make_the_report_fail() {
    let report = Report::from_findings(vec![Finding::error(
        "CFG-001",
        "contract",
        "contract",
        "invalid contract",
    )]);
    assert_eq!(report.status, ReportStatus::Fail);
    assert_eq!(report.summary.errors, 1);
}

#[test]
fn json_ends_with_a_newline() {
    let json = Report::from_findings(Vec::new())
        .json()
        .expect("serialize report");
    assert!(json.ends_with('\n'));
}

#[test]
fn bounded_reports_keep_exact_totals_and_groups() {
    let report = Report::from_findings_with_limit(
        [
            Finding::error("CFG-001", "contract", "contract", "first"),
            Finding::error("CFG-001", "contract", "contract", "second"),
            Finding::error("NOTE-001", "inventory", "source", "third")
                .with_severity(Severity::Note),
        ],
        DiagnosticLimit::Bounded(1),
    );

    assert_eq!(report.schema, 3);
    assert_eq!(report.summary.errors, 2);
    assert_eq!(report.summary.notes, 1);
    assert_eq!(report.summary.retained, 1);
    assert_eq!(report.summary.omitted, 2);
    assert!(report.truncated);
    assert_eq!(report.groups[0].count, 2);
    assert_eq!(report.status, ReportStatus::Fail);
    let json: serde_json::Value =
        serde_json::from_str(&report.json().expect("render JSON")).expect("parse JSON");
    assert_eq!(json["schema"], 3);
    assert_eq!(json["summary"]["errors"], 2);
    assert_eq!(json["summary"]["retained"], 1);
    assert_eq!(json["summary"]["omitted"], 2);
    assert_eq!(json["truncated"], true);
    assert_eq!(json["max_findings"], 1);
    assert_eq!(json["analysis"]["complete"], true);
    assert_eq!(json["groups"][0]["count"], 2);
    let human = report.human();
    assert!(human.contains("Diagnostics: 3 total; showing 1."));
    let group = human
        .lines()
        .find(|line| line.trim_start().starts_with("CFG-001"))
        .expect("render grouped diagnostic");
    assert!(group.contains("contract"));
    assert!(group.contains("error"));
    assert!(group.ends_with('2'));
    assert!(human.ends_with("Status: fail (2 errors, 0 warnings, 1 notes)\n"));
}

#[test]
fn zero_limit_is_aggregate_only_and_all_retains_every_finding() {
    let findings = [
        Finding::error("ONE", "rule", "test", "one"),
        Finding::error("TWO", "rule", "test", "two"),
    ];
    let aggregate = Report::from_findings_with_limit(findings.clone(), DiagnosticLimit::Bounded(0));
    let complete = Report::from_findings_with_limit(findings, DiagnosticLimit::All);

    assert!(aggregate.findings.is_empty());
    assert_eq!(aggregate.summary.omitted, 2);
    assert_eq!(aggregate.status, ReportStatus::Fail);
    assert_eq!(complete.findings.len(), 2);
    assert_eq!(complete.summary.omitted, 0);
    assert!(!complete.truncated);
}

#[test]
fn appended_review_findings_remain_authoritative_when_not_retained() {
    let report =
        Report::from_findings_with_limit(Vec::new(), DiagnosticLimit::Bounded(0)).with_findings([
            Finding::error("REVIEW-001", "review.lock", "review", "missing lock"),
        ]);

    assert_eq!(report.status, ReportStatus::Fail);
    assert_eq!(report.summary.errors, 1);
    assert_eq!(report.summary.retained, 0);
    assert_eq!(report.groups[0].id, "REVIEW-001");
}
