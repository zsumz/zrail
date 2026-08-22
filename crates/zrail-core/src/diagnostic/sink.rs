//! Finding retention is configurable while aggregate counts remain exact.

use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error as _};

use super::{Finding, Severity};

/// Default number of individual findings retained for display.
pub const MAX_REPORT_FINDINGS: usize = 10_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Maximum individual diagnostics retained in a report.
pub enum DiagnosticLimit {
    /// Retain at most this many individual findings while counting all findings.
    Bounded(usize),
    /// Retain every finding allowed by the repository's input safety limits.
    All,
}

impl Default for DiagnosticLimit {
    fn default() -> Self {
        Self::Bounded(MAX_REPORT_FINDINGS)
    }
}

impl Serialize for DiagnosticLimit {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Bounded(limit) => {
                serializer.serialize_u64(u64::try_from(*limit).map_err(serde::ser::Error::custom)?)
            }
            Self::All => serializer.serialize_str("all"),
        }
    }
}

impl<'de> Deserialize<'de> for DiagnosticLimit {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Wire {
            Bounded(usize),
            Name(String),
        }

        match Wire::deserialize(deserializer)? {
            Wire::Bounded(limit) => Ok(Self::Bounded(limit)),
            Wire::Name(name) if name == "all" => Ok(Self::All),
            Wire::Name(name) => Err(D::Error::custom(format!(
                "unsupported diagnostic limit {name:?}"
            ))),
        }
    }
}

impl DiagnosticLimit {
    pub(crate) const fn retains(self, retained: usize) -> bool {
        match self {
            Self::Bounded(limit) => retained < limit,
            Self::All => true,
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
/// Stable aggregate identity for one diagnostic rule and severity.
pub struct FindingGroup {
    /// Stable diagnostic identifier.
    pub id: String,
    /// Contract rail or analysis rule that produced the finding.
    pub rule: String,
    /// Failure impact shared by findings in this group.
    pub severity: Severity,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
/// Exact finding counts retained independently of individual report payloads.
pub struct FindingTotals {
    /// Counts by failure impact.
    pub severity: BTreeMap<Severity, usize>,
    /// Counts by stable diagnostic identifier, rule, and severity.
    pub groups: BTreeMap<FindingGroup, usize>,
}

impl FindingTotals {
    pub(crate) fn record(&mut self, finding: &Finding) {
        *self.severity.entry(finding.severity).or_default() += 1;
        *self
            .groups
            .entry(FindingGroup {
                id: finding.id.clone(),
                rule: finding.rule.clone(),
                severity: finding.severity,
            })
            .or_default() += 1;
    }

    pub(crate) fn total(&self) -> usize {
        self.severity.values().sum()
    }
}

#[derive(Debug)]
/// Insertion-order collector with bounded payloads and exact aggregate counts.
pub struct FindingSink {
    retained: Vec<Finding>,
    totals: FindingTotals,
    limit: DiagnosticLimit,
}

impl Default for FindingSink {
    fn default() -> Self {
        Self::with_limit(DiagnosticLimit::default())
    }
}

impl FindingSink {
    /// Creates an empty collector with an explicit display-retention limit.
    pub fn with_limit(limit: DiagnosticLimit) -> Self {
        Self {
            retained: Vec::new(),
            totals: FindingTotals::default(),
            limit,
        }
    }

    /// Collects findings with the default 10,000-item display limit.
    pub fn from_findings(findings: impl IntoIterator<Item = Finding>) -> Self {
        Self::from_findings_with_limit(findings, DiagnosticLimit::default())
    }

    /// Collects findings with an explicit display-retention limit.
    pub fn from_findings_with_limit(
        findings: impl IntoIterator<Item = Finding>,
        limit: DiagnosticLimit,
    ) -> Self {
        let mut sink = Self::with_limit(limit);
        for finding in findings {
            sink.push(finding);
        }
        sink
    }

    /// Counts every finding and retains it only while the display limit permits.
    pub fn push(&mut self, finding: Finding) {
        self.totals.record(&finding);
        if self.limit.retains(self.retained.len()) {
            self.retained.push(finding);
        }
    }

    /// Iterates retained findings in insertion order.
    pub fn iter(&self) -> impl Iterator<Item = &Finding> {
        self.retained.iter()
    }

    /// Returns exact totals including findings omitted from the payload.
    pub const fn totals(&self) -> &FindingTotals {
        &self.totals
    }

    /// Returns the configured payload-retention limit.
    pub const fn limit(&self) -> DiagnosticLimit {
        self.limit
    }

    /// Finalizes the retained individual findings without a synthetic sentinel.
    pub fn into_findings(self) -> Vec<Finding> {
        self.retained
    }

    pub(crate) fn into_parts(self) -> (Vec<Finding>, FindingTotals, DiagnosticLimit) {
        (self.retained, self.totals, self.limit)
    }
}
