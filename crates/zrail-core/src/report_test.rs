//! Report status and deterministic JSON.

use crate::diagnostic::Finding;

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
