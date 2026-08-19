//! Stable, teachable diagnostics shared by every adapter and output format.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
/// Impact assigned to an architecture finding.
pub enum Severity {
    /// A violated rail that makes the analysis fail.
    Error,
    /// A non-failing condition that deserves attention.
    Warning,
    /// Informational context with no failure implication.
    Note,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
/// Confidence and completeness of the analysis producing a finding.
pub enum AnalysisQuality {
    /// The adapter resolved the relevant source relationship exactly.
    Exact,
    /// The adapter used a sound over-approximation that may report extra matches.
    Conservative,
    /// The adapter could not establish the relevant relationship.
    Unresolved,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// Caller-supplied start and end coordinates within a source file.
/// Coordinates are not normalized or range-checked by `zrail-core`.
pub struct SourceSpan {
    /// Starting line number in the adapter's coordinate convention.
    pub line: usize,
    /// Starting column number in the adapter's coordinate convention.
    pub column: usize,
    /// Ending line number in the adapter's coordinate convention.
    pub end_line: usize,
    /// Ending column number in the adapter's coordinate convention.
    pub end_column: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// Stable architecture diagnostic shared by adapters and report formats.
pub struct Finding {
    /// Stable diagnostic identity, such as `ZR-LIMIT-001`.
    pub id: String,
    /// Contract rail or analysis rule that produced the finding.
    pub rule: String,
    /// Broad diagnostic grouping used by consumers.
    pub category: String,
    /// Failure impact of this finding.
    pub severity: Severity,
    /// Human-readable description of the observed condition.
    pub message: String,
    /// Normalized repository-relative source path, when applicable.
    pub path: Option<String>,
    /// Source coordinates within `path`, when available.
    pub span: Option<SourceSpan>,
    /// Contract-authored justification relevant to the rail, when available.
    pub reason: Option<String>,
    /// Suggested remediation, when the adapter can provide one.
    pub help: Option<String>,
    /// Confidence and completeness of the producing analysis.
    pub analysis: AnalysisQuality,
    /// Lowercase SHA-256 identity derived from id, rule, path, message, and span.
    pub fingerprint: String,
}

/// Maximum findings retained in a finalized bounded sink, including its sentinel.
pub const MAX_REPORT_FINDINGS: usize = 10_000;

#[derive(Debug, Default)]
/// Bounded insertion-order collector that reports omitted diagnostics explicitly.
pub struct FindingSink {
    findings: Vec<Finding>,
    omitted: usize,
}

impl FindingSink {
    /// Collects findings in iteration order while applying the reporting limit.
    pub fn from_findings(findings: impl IntoIterator<Item = Finding>) -> Self {
        let mut sink = Self::default();
        for finding in findings {
            sink.push(finding);
        }
        sink
    }

    /// Appends a finding or records it as omitted when the bounded payload is full.
    /// One slot is reserved for the unresolved sentinel added by [`FindingSink::into_findings`].
    pub fn push(&mut self, finding: Finding) {
        if self.findings.len() < MAX_REPORT_FINDINGS - 1 {
            self.findings.push(finding);
        } else {
            self.omitted += 1;
        }
    }

    /// Iterates retained findings in insertion order, excluding any future sentinel.
    pub fn iter(&self) -> impl Iterator<Item = &Finding> {
        self.findings.iter()
    }

    /// Finalizes the retained findings in insertion order.
    /// If anything was omitted, the final item is unresolved `ZR-LIMIT-001` with the exact count.
    pub fn into_findings(mut self) -> Vec<Finding> {
        if self.omitted > 0 {
            self.findings.push(
                Finding::error(
                    "ZR-LIMIT-001",
                    "analysis.diagnostic-limit",
                    "analysis",
                    format!(
                        "diagnostic safety limit reached; {} additional findings omitted",
                        self.omitted
                    ),
                )
                .with_analysis(AnalysisQuality::Unresolved)
                .with_help("reduce repository scope or fix the first reported findings"),
            );
        }
        self.findings
    }
}

impl Finding {
    /// Creates an exact error finding without source location or optional context.
    /// Its fingerprint covers the supplied identity, rule, message, and empty location.
    pub fn error(
        id: impl Into<String>,
        rule: impl Into<String>,
        category: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        let mut finding = Self {
            id: id.into(),
            rule: rule.into(),
            category: category.into(),
            severity: Severity::Error,
            message: message.into(),
            path: None,
            span: None,
            reason: None,
            help: None,
            analysis: AnalysisQuality::Exact,
            fingerprint: String::new(),
        };
        finding.refresh_fingerprint();
        finding
    }

    #[must_use]
    /// Sets the repository-relative path and optional span, then refreshes identity.
    pub fn at(mut self, path: impl Into<String>, span: Option<SourceSpan>) -> Self {
        self.path = Some(path.into());
        self.span = span;
        self.refresh_fingerprint();
        self
    }

    #[must_use]
    /// Attaches contract-authored justification and preserves fingerprint validity.
    pub fn because(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self.refresh_fingerprint();
        self
    }

    #[must_use]
    /// Attaches remediation guidance and preserves fingerprint validity.
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self.refresh_fingerprint();
        self
    }

    #[must_use]
    /// Reclassifies analysis quality and preserves fingerprint validity.
    pub fn with_analysis(mut self, analysis: AnalysisQuality) -> Self {
        self.analysis = analysis;
        self.refresh_fingerprint();
        self
    }

    #[must_use]
    /// Reclassifies failure impact and preserves fingerprint validity.
    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self.refresh_fingerprint();
        self
    }

    fn refresh_fingerprint(&mut self) {
        let mut digest = Sha256::new();
        for value in [
            self.id.as_str(),
            self.rule.as_str(),
            self.path.as_deref().unwrap_or(""),
            self.message.as_str(),
        ] {
            digest.update(value.as_bytes());
            digest.update([0]);
        }
        if let Some(span) = self.span {
            digest.update(span.line.to_le_bytes());
            digest.update(span.column.to_le_bytes());
            digest.update(span.end_line.to_le_bytes());
            digest.update(span.end_column.to_le_bytes());
        }
        self.fingerprint = format!("{:x}", digest.finalize());
    }
}

/// Sorts findings deterministically by path, start position, id, then message.
pub fn sort_findings(findings: &mut [Finding]) {
    findings.sort_by(|left, right| {
        (
            left.path.as_deref().unwrap_or(""),
            left.span.map_or(0, |span| span.line),
            left.span.map_or(0, |span| span.column),
            left.id.as_str(),
            left.message.as_str(),
        )
            .cmp(&(
                right.path.as_deref().unwrap_or(""),
                right.span.map_or(0, |span| span.line),
                right.span.map_or(0, |span| span.column),
                right.id.as_str(),
                right.message.as_str(),
            ))
    });
}

#[cfg(test)]
#[path = "diagnostic_test.rs"]
mod diagnostic_test;
