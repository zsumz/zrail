//! Public semantic-diff vocabulary.

use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
/// Semantic impact assigned to one architecture change.
pub enum ChangeKind {
    /// Policy authority was broadened or a hard limit was weakened.
    Grant,
    /// Policy authority was narrowed or a hard limit was tightened.
    Revoke,
    /// An advisory target or measured ratchet moved in a weaker direction.
    Debt,
    /// An advisory target or measured ratchet moved in a stronger direction.
    Cleanup,
    /// Architecture state changed without changing effective authority.
    Neutral,
    /// Effective authority cannot be established from the supplied state.
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// One deterministic semantic change to an architecture rail.
pub struct ArchitectureChange {
    /// Authority or debt impact of the change.
    pub kind: ChangeKind,
    /// Stable contract or lock rail identity.
    pub rail: String,
    /// Stable identity of the governed package, rule, path, or authority state.
    pub subject: String,
    /// Human-readable explanation of the semantic change.
    pub message: String,
    /// Canonical prior value, when the change compares values.
    pub before: Option<String>,
    /// Canonical new value, when the change compares values.
    pub after: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// Counts of semantic changes grouped by impact.
pub struct DiffSummary {
    /// Number of authority-broadening changes.
    pub grants: usize,
    /// Number of authority-tightening changes.
    pub revokes: usize,
    /// Number of debt-increasing changes.
    pub debt: usize,
    /// Number of debt-reducing changes.
    pub cleanup: usize,
    /// Number of authority-neutral changes.
    pub neutral: usize,
    /// Number of changes whose authority could not be established.
    pub unknown: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// Deterministically ordered semantic architecture diff.
pub struct DiffReport {
    /// Diff wire-format version; currently `1`.
    pub schema: u64,
    /// Impact counts derived from `changes`.
    pub summary: DiffSummary,
    /// Changes sorted by kind, rail, subject, then message.
    pub changes: Vec<ArchitectureChange>,
}

impl ArchitectureChange {
    pub(super) fn new(
        kind: ChangeKind,
        rail: impl Into<String>,
        subject: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            rail: rail.into(),
            subject: subject.into(),
            message: message.into(),
            before: None,
            after: None,
        }
    }

    pub(super) fn values(mut self, before: impl Into<String>, after: impl Into<String>) -> Self {
        self.before = Some(before.into());
        self.after = Some(after.into());
        self
    }
}

impl DiffReport {
    pub(super) fn new(mut changes: Vec<ArchitectureChange>) -> Self {
        changes.sort_by(|left, right| {
            (&left.kind, &left.rail, &left.subject, &left.message).cmp(&(
                &right.kind,
                &right.rail,
                &right.subject,
                &right.message,
            ))
        });
        let mut summary = DiffSummary::default();
        for change in &changes {
            match change.kind {
                ChangeKind::Grant => summary.grants += 1,
                ChangeKind::Revoke => summary.revokes += 1,
                ChangeKind::Debt => summary.debt += 1,
                ChangeKind::Cleanup => summary.cleanup += 1,
                ChangeKind::Neutral => summary.neutral += 1,
                ChangeKind::Unknown => summary.unknown += 1,
            }
        }
        Self {
            schema: 1,
            summary,
            changes,
        }
    }

    /// Returns `true` when automation must fail closed.
    ///
    /// Grants, increased debt, and unknown authority deny acceptance; revokes,
    /// cleanup, and neutral changes do not.
    pub fn denies_grants(&self) -> bool {
        self.summary.grants > 0 || self.summary.debt > 0 || self.summary.unknown > 0
    }

    /// Serializes the diff as pretty JSON terminated by one newline.
    pub fn json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self).map(|mut value| {
            value.push('\n');
            value
        })
    }

    /// Renders stored changes followed by a one-line impact summary.
    pub fn human(&self) -> String {
        let mut output = String::new();
        for change in &self.changes {
            let _ = write!(
                output,
                "{} {} {}\n  {}\n",
                kind_name(change.kind),
                change.rail,
                change.subject,
                change.message
            );
            if let (Some(before), Some(after)) = (&change.before, &change.after) {
                let _ = writeln!(output, "  before: {before}\n  after:  {after}");
            }
            output.push('\n');
        }
        let _ = writeln!(
            output,
            "Changes: {} grants, {} revokes, {} debt, {} cleanup, {} neutral, {} unknown",
            self.summary.grants,
            self.summary.revokes,
            self.summary.debt,
            self.summary.cleanup,
            self.summary.neutral,
            self.summary.unknown
        );
        output
    }
}

#[cfg(test)]
#[path = "model_test.rs"]
mod model_test;

const fn kind_name(kind: ChangeKind) -> &'static str {
    match kind {
        ChangeKind::Grant => "GRANT",
        ChangeKind::Revoke => "REVOKE",
        ChangeKind::Debt => "DEBT",
        ChangeKind::Cleanup => "CLEANUP",
        ChangeKind::Neutral => "NEUTRAL",
        ChangeKind::Unknown => "UNKNOWN",
    }
}
