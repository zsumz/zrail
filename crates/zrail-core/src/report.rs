//! Deterministic human and machine reports.

use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

use crate::diagnostic::{Finding, Severity, sort_findings};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
/// Overall result of repository architecture analysis.
pub enum ReportStatus {
    /// Analysis completed with no error-severity findings.
    Pass,
    /// Analysis completed with one or more error-severity findings.
    Fail,
    /// Analysis could not produce a valid architecture result.
    Invalid,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// Finding counts grouped by failure impact.
pub struct ReportSummary {
    /// Number of error-severity findings.
    pub errors: usize,
    /// Number of warning-severity findings.
    pub warnings: usize,
    /// Number of note-severity findings.
    pub notes: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// Deterministic machine- and human-readable architecture analysis result.
pub struct Report {
    /// Report wire-format version; currently `1`.
    pub schema: u64,
    /// Overall pass, fail, or invalid state.
    pub status: ReportStatus,
    /// Severity counts derived from `findings`.
    pub summary: ReportSummary,
    /// Diagnostics in deterministic source order.
    pub findings: Vec<Finding>,
}

impl Report {
    /// Builds a schema-1 report from findings sorted deterministically in place.
    ///
    /// The status is `Pass` exactly when no finding has [`Severity::Error`]; this
    /// constructor never emits `Invalid`.
    pub fn from_findings(mut findings: Vec<Finding>) -> Self {
        sort_findings(&mut findings);
        let summary = ReportSummary {
            errors: findings
                .iter()
                .filter(|finding| finding.severity == Severity::Error)
                .count(),
            warnings: findings
                .iter()
                .filter(|finding| finding.severity == Severity::Warning)
                .count(),
            notes: findings
                .iter()
                .filter(|finding| finding.severity == Severity::Note)
                .count(),
        };
        Self {
            schema: 1,
            status: if summary.errors == 0 {
                ReportStatus::Pass
            } else {
                ReportStatus::Fail
            },
            summary,
            findings,
        }
    }

    /// Serializes the report as pretty JSON terminated by one newline.
    pub fn json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self).map(|mut value| {
            value.push('\n');
            value
        })
    }

    /// Renders findings and the summary in their stored deterministic order.
    pub fn human(&self) -> String {
        let mut output = String::new();
        for finding in &self.findings {
            let severity = match finding.severity {
                Severity::Error => "error",
                Severity::Warning => "warning",
                Severity::Note => "note",
            };
            let _ = writeln!(output, "{severity}[{}]: {}", finding.id, finding.message);
            if let Some(path) = &finding.path {
                if let Some(span) = finding.span {
                    let _ = writeln!(output, "  --> {path}:{}:{}", span.line, span.column);
                } else {
                    let _ = writeln!(output, "  --> {path}");
                }
            }
            let _ = writeln!(output, "   = rule: {}", finding.rule);
            let _ = writeln!(output, "   = analysis: {}", analysis_name(finding.analysis));
            if let Some(reason) = &finding.reason {
                let _ = writeln!(output, "   = reason: {reason}");
            }
            if let Some(help) = &finding.help {
                let _ = writeln!(output, "   = help: {help}");
            }
            output.push('\n');
        }
        let _ = writeln!(
            output,
            "Status: {} ({} errors, {} warnings, {} notes)",
            status_name(self.status),
            self.summary.errors,
            self.summary.warnings,
            self.summary.notes
        );
        output
    }
}

const fn analysis_name(quality: crate::AnalysisQuality) -> &'static str {
    match quality {
        crate::AnalysisQuality::Exact => "exact",
        crate::AnalysisQuality::Conservative => "conservative",
        crate::AnalysisQuality::Unresolved => "unresolved",
    }
}

const fn status_name(status: ReportStatus) -> &'static str {
    match status {
        ReportStatus::Pass => "pass",
        ReportStatus::Fail => "fail",
        ReportStatus::Invalid => "invalid",
    }
}

#[cfg(test)]
#[path = "report_test.rs"]
mod report_test;
