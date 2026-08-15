//! Public semantic-diff vocabulary.

use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ChangeKind {
    Grant,
    Revoke,
    Debt,
    Cleanup,
    Neutral,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArchitectureChange {
    pub kind: ChangeKind,
    pub rail: String,
    pub subject: String,
    pub message: String,
    pub before: Option<String>,
    pub after: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DiffSummary {
    pub grants: usize,
    pub revokes: usize,
    pub debt: usize,
    pub cleanup: usize,
    pub neutral: usize,
    pub unknown: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DiffReport {
    pub schema: u64,
    pub summary: DiffSummary,
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

    pub fn denies_grants(&self) -> bool {
        self.summary.grants > 0 || self.summary.debt > 0 || self.summary.unknown > 0
    }

    pub fn json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self).map(|mut value| {
            value.push('\n');
            value
        })
    }

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
            "Changes: {} grants, {} revokes, {} debt, {} cleanup, {} unknown",
            self.summary.grants,
            self.summary.revokes,
            self.summary.debt,
            self.summary.cleanup,
            self.summary.unknown
        );
        output
    }
}

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
