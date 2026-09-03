//! Scoped review of a supported prior lock epoch against current semantics.

mod bridge;
mod surfaces;

use std::{collections::BTreeSet, error::Error, fmt};

use serde::{Deserialize, Serialize};

use crate::{LOCK_SCHEMA, LOCK_SEMANTICS, LockFile};
pub use bridge::{
    LockMigrationBridgeReport, LockMigrationFileChange, LockMigrationFileState,
    LockMigrationRevision,
};
use surfaces::surfaces;

const SUPPORTED_PRIOR_EPOCHS: &[(u64, u64)] = &[(1, 1), (1, 2), (2, 3), (3, 4), (3, 5)];

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
/// Relationship between one old lock subject and its new interpretation.
pub enum LockMigrationClassification {
    /// The exact authority subject and value retain the same meaning.
    Preserved,
    /// The old engine recorded authority that the new engine no longer observes.
    Retired,
    /// The new engine observes an exact subject absent from the old epoch.
    NewlyObservable,
    /// Both epochs observe the subject but resolve different exact authority.
    ChangedInterpretation,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// One scoped lock subject in an epoch migration review.
pub struct LockMigrationEntry {
    /// Migration classification for this exact subject.
    pub classification: LockMigrationClassification,
    /// Stable authority rail.
    pub rail: String,
    /// Stable package, path, rule, or resolved-edge identity.
    pub subject: String,
    /// Canonical prior value, when the old epoch observed the subject.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,
    /// Canonical new value, when the new epoch observes the subject.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// Exact counts derived from migration entries.
pub struct LockMigrationSummary {
    /// Subjects whose exact authority is preserved.
    pub preserved: usize,
    /// Subjects no longer observed by the new epoch.
    pub retired: usize,
    /// Subjects made newly observable by the new epoch.
    pub newly_observable: usize,
    /// Subjects whose exact interpretation changed.
    pub changed_interpretation: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// Deterministic prior-epoch migration report.
pub struct LockMigrationReport {
    /// Migration report schema; currently `1`.
    pub schema: u64,
    /// Prior lock semantics decoded by the adapter.
    pub from_semantics: u64,
    /// New lock semantics produced for the reviewed comparison revision.
    pub to_semantics: u64,
    /// Counts derived from the complete subject list.
    pub summary: LockMigrationSummary,
    /// Every old or new exact authority subject in stable order.
    pub entries: Vec<LockMigrationEntry>,
}

impl LockMigrationReport {
    /// Canonical JSON bytes used as explicit migration-acceptance identity.
    pub fn json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self).map(|mut value| {
            value.push('\n');
            value
        })
    }

    /// Lowercase SHA-256 identity of the compact canonical report payload.
    /// Returns an empty identity if serialization cannot be produced.
    pub fn sha256(&self) -> String {
        let Ok(bytes) = serde_json::to_vec(self) else {
            return String::new();
        };
        crate::sha256_hex(&bytes)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Unsupported or mismatched lock migration state.
pub struct LockMigrationError(String);

impl fmt::Display for LockMigrationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for LockMigrationError {}

/// Compares old authority with a new-engine lock built from the identical revision.
pub fn compare_lock_epochs(
    before: &LockFile,
    after: &LockFile,
) -> Result<LockMigrationReport, LockMigrationError> {
    compare_epochs(before, after, true)
}

/// Compares old authority with a current-engine lock from a reviewed descendant revision.
///
/// Unlike [`compare_lock_epochs`], this bridge permits the contract digest to change. The
/// caller must bind both revisions, both locks, and the complete repository change manifest
/// in a [`LockMigrationBridgeReport`].
pub fn compare_lock_epochs_across_revisions(
    before: &LockFile,
    after: &LockFile,
) -> Result<LockMigrationReport, LockMigrationError> {
    compare_epochs(before, after, false)
}

fn compare_epochs(
    before: &LockFile,
    after: &LockFile,
    require_identical_contract: bool,
) -> Result<LockMigrationReport, LockMigrationError> {
    if !SUPPORTED_PRIOR_EPOCHS.contains(&(before.schema, before.semantics))
        || after.schema != LOCK_SCHEMA
        || after.semantics != LOCK_SEMANTICS
    {
        return Err(LockMigrationError(format!(
            "lock migration supports released epochs schema 1/semantics 1, schema 1/semantics 2, schema 2/semantics 3, schema 3/semantics 4, and schema 3/semantics 5 to schema {LOCK_SCHEMA}/semantics {LOCK_SEMANTICS}"
        )));
    }
    if require_identical_contract && before.contract_sha256 != after.contract_sha256 {
        return Err(LockMigrationError(
            "lock migration requires identical contract bytes at the base revision".into(),
        ));
    }
    let old = surfaces(before).map_err(|error| {
        LockMigrationError(format!("serialize prior lock migration surfaces: {error}"))
    })?;
    let new = surfaces(after).map_err(|error| {
        LockMigrationError(format!(
            "serialize current lock migration surfaces: {error}"
        ))
    })?;
    let keys = old
        .keys()
        .chain(new.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    let entries = keys
        .into_iter()
        .filter_map(|(rail, subject)| {
            let before = old.get(&(rail.clone(), subject.clone())).cloned();
            let after = new.get(&(rail.clone(), subject.clone())).cloned();
            let classification = match (&before, &after) {
                (Some(left), Some(right)) if left == right => {
                    LockMigrationClassification::Preserved
                }
                (Some(_), Some(_)) => LockMigrationClassification::ChangedInterpretation,
                (Some(_), None) => LockMigrationClassification::Retired,
                (None, Some(_)) => LockMigrationClassification::NewlyObservable,
                (None, None) => return None,
            };
            Some(LockMigrationEntry {
                classification,
                rail,
                subject,
                before,
                after,
            })
        })
        .collect::<Vec<_>>();
    let mut summary = LockMigrationSummary::default();
    for entry in &entries {
        match entry.classification {
            LockMigrationClassification::Preserved => summary.preserved += 1,
            LockMigrationClassification::Retired => summary.retired += 1,
            LockMigrationClassification::NewlyObservable => summary.newly_observable += 1,
            LockMigrationClassification::ChangedInterpretation => {
                summary.changed_interpretation += 1;
            }
        }
    }
    Ok(LockMigrationReport {
        schema: 1,
        from_semantics: before.semantics,
        to_semantics: after.semantics,
        summary,
        entries,
    })
}

#[cfg(test)]
#[path = "migration_test.rs"]
mod migration_test;
