//! Stable, teachable diagnostics shared by every adapter and output format.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Severity {
    Error,
    Warning,
    Note,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AnalysisQuality {
    Exact,
    Conservative,
    Unresolved,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SourceSpan {
    pub line: usize,
    pub column: usize,
    pub end_line: usize,
    pub end_column: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Finding {
    pub id: String,
    pub rule: String,
    pub category: String,
    pub severity: Severity,
    pub message: String,
    pub path: Option<String>,
    pub span: Option<SourceSpan>,
    pub reason: Option<String>,
    pub help: Option<String>,
    pub analysis: AnalysisQuality,
    pub fingerprint: String,
}

pub const MAX_REPORT_FINDINGS: usize = 10_000;

#[derive(Debug, Default)]
pub struct FindingSink {
    findings: Vec<Finding>,
    omitted: usize,
}

impl FindingSink {
    pub fn from_findings(findings: impl IntoIterator<Item = Finding>) -> Self {
        let mut sink = Self::default();
        for finding in findings {
            sink.push(finding);
        }
        sink
    }

    pub fn push(&mut self, finding: Finding) {
        if self.findings.len() < MAX_REPORT_FINDINGS - 1 {
            self.findings.push(finding);
        } else {
            self.omitted += 1;
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Finding> {
        self.findings.iter()
    }

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
    pub fn at(mut self, path: impl Into<String>, span: Option<SourceSpan>) -> Self {
        self.path = Some(path.into());
        self.span = span;
        self.refresh_fingerprint();
        self
    }

    #[must_use]
    pub fn because(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self.refresh_fingerprint();
        self
    }

    #[must_use]
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self.refresh_fingerprint();
        self
    }

    #[must_use]
    pub fn with_analysis(mut self, analysis: AnalysisQuality) -> Self {
        self.analysis = analysis;
        self.refresh_fingerprint();
        self
    }

    #[must_use]
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
