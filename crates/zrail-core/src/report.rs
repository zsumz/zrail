//! Deterministic human and machine reports.

mod render;

use serde::{Deserialize, Serialize};

use crate::diagnostic::{
    DiagnosticLimit, Finding, FindingSink, FindingTotals, Severity, sort_findings,
};

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
/// Exact finding counts and payload-retention totals.
pub struct ReportSummary {
    /// Number of error-severity findings.
    pub errors: usize,
    /// Number of warning-severity findings.
    pub warnings: usize,
    /// Number of note-severity findings.
    pub notes: usize,
    /// Number of individual findings retained in the report payload.
    pub retained: usize,
    /// Number of findings counted but omitted from the report payload.
    pub omitted: usize,
}

impl ReportSummary {
    pub(crate) const fn total(self) -> usize {
        self.errors + self.warnings + self.notes
    }

    fn record(&mut self, severity: Severity, retained: bool) {
        match severity {
            Severity::Error => self.errors += 1,
            Severity::Warning => self.warnings += 1,
            Severity::Note => self.notes += 1,
        }
        if retained {
            self.retained += 1;
        } else {
            self.omitted += 1;
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// Exact count for one diagnostic identity, rule, and severity.
pub struct ReportGroup {
    /// Stable diagnostic identifier.
    pub id: String,
    /// Contract rail or analysis rule that produced the finding.
    pub rule: String,
    /// Failure impact shared by findings in this group.
    pub severity: Severity,
    /// Exact number of matching findings, including omitted payloads.
    pub count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// Deterministic machine- and human-readable architecture analysis result.
pub struct Report {
    /// Report wire-format version; currently `2`.
    pub schema: u64,
    /// Overall pass, fail, or invalid state.
    pub status: ReportStatus,
    /// Exact severity and payload-retention counts.
    pub summary: ReportSummary,
    /// Whether individual findings were omitted from the report payload.
    pub truncated: bool,
    /// Configured individual-finding retention limit.
    pub limit: DiagnosticLimit,
    /// Exact aggregate counts by diagnostic identity, rule, and severity.
    pub groups: Vec<ReportGroup>,
    /// Retained diagnostics in deterministic source order.
    pub findings: Vec<Finding>,
}

impl Report {
    /// Builds a schema-2 report with the default 10,000-finding payload limit.
    pub fn from_findings(findings: impl IntoIterator<Item = Finding>) -> Self {
        Self::from_sink(FindingSink::from_findings(findings))
    }

    /// Builds a schema-2 report with an explicit individual-finding payload limit.
    pub fn from_findings_with_limit(
        findings: impl IntoIterator<Item = Finding>,
        limit: DiagnosticLimit,
    ) -> Self {
        Self::from_sink(FindingSink::from_findings_with_limit(findings, limit))
    }

    /// Builds a schema-2 report from a collector that already holds exact totals.
    pub fn from_sink(sink: FindingSink) -> Self {
        let (mut findings, totals, limit) = sink.into_parts();
        sort_findings(&mut findings);
        let summary = summary_from_totals(&totals, findings.len());
        Self {
            schema: 2,
            status: status_from_summary(summary),
            summary,
            truncated: summary.omitted > 0,
            limit,
            groups: groups_from_totals(totals),
            findings,
        }
    }

    /// Adds findings while preserving exact pre-existing aggregate counts.
    #[must_use]
    pub fn with_findings(mut self, findings: impl IntoIterator<Item = Finding>) -> Self {
        for finding in findings {
            let retained = self.limit.retains(self.findings.len());
            self.summary.record(finding.severity, retained);
            self.record_group(&finding);
            if retained {
                self.findings.push(finding);
            }
        }
        sort_findings(&mut self.findings);
        self.groups.sort_by(group_order);
        self.truncated = self.summary.omitted > 0;
        self.status = status_from_summary(self.summary);
        self
    }

    fn record_group(&mut self, finding: &Finding) {
        if let Some(group) = self.groups.iter_mut().find(|group| {
            group.id == finding.id
                && group.rule == finding.rule
                && group.severity == finding.severity
        }) {
            group.count += 1;
        } else {
            self.groups.push(ReportGroup {
                id: finding.id.clone(),
                rule: finding.rule.clone(),
                severity: finding.severity,
                count: 1,
            });
        }
    }

    /// Serializes the report as pretty JSON terminated by one newline.
    pub fn json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self).map(|mut value| {
            value.push('\n');
            value
        })
    }

    /// Renders exact aggregates and retained findings in deterministic order.
    pub fn human(&self) -> String {
        render::human(self)
    }
}

fn summary_from_totals(totals: &FindingTotals, retained: usize) -> ReportSummary {
    let total = totals.total();
    ReportSummary {
        errors: totals.severity.get(&Severity::Error).copied().unwrap_or(0),
        warnings: totals
            .severity
            .get(&Severity::Warning)
            .copied()
            .unwrap_or(0),
        notes: totals.severity.get(&Severity::Note).copied().unwrap_or(0),
        retained,
        omitted: total.saturating_sub(retained),
    }
}

fn groups_from_totals(totals: FindingTotals) -> Vec<ReportGroup> {
    totals
        .groups
        .into_iter()
        .map(|(group, count)| ReportGroup {
            id: group.id,
            rule: group.rule,
            severity: group.severity,
            count,
        })
        .collect()
}

const fn status_from_summary(summary: ReportSummary) -> ReportStatus {
    if summary.errors == 0 {
        ReportStatus::Pass
    } else {
        ReportStatus::Fail
    }
}

fn group_order(left: &ReportGroup, right: &ReportGroup) -> std::cmp::Ordering {
    (&left.id, &left.rule, left.severity).cmp(&(&right.id, &right.rule, right.severity))
}

#[cfg(test)]
#[path = "report_test.rs"]
mod report_test;
