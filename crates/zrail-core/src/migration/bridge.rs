//! Digest-bound identity for a reviewed migration across committed revisions.

use serde::{Deserialize, Serialize};

use super::LockMigrationReport;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// One immutable revision and its exact contract and lock identities.
pub struct LockMigrationRevision {
    /// Full Git commit identifier.
    pub commit: String,
    /// SHA-256 identity of the complete loaded contract.
    pub contract_sha256: String,
    /// SHA-256 identity of the canonical lock bytes.
    pub lock_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// Content identity of one regular file at one side of a migration bridge.
pub struct LockMigrationFileState {
    /// Canonical Git file mode.
    pub mode: String,
    /// SHA-256 identity of the file bytes.
    pub sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// One added, removed, content-changed, or mode-changed repository file.
pub struct LockMigrationFileChange {
    /// Normalized repository-relative path.
    pub path: String,
    /// File identity at the base revision, if present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub before: Option<LockMigrationFileState>,
    /// File identity at the target revision, if present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub after: Option<LockMigrationFileState>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// Reviewed bridge from an unanalyzable prior-epoch commit to an analyzable descendant.
pub struct LockMigrationBridgeReport {
    /// Bridge report schema; currently `1`.
    pub schema: u64,
    /// Exact prior-epoch authority revision.
    pub base: LockMigrationRevision,
    /// Exact current-engine target revision.
    pub target: LockMigrationRevision,
    /// Stable current-engine failure observed while reanalyzing the base.
    pub base_analysis_error: String,
    /// Complete, canonically ordered repository file changes between the revisions.
    pub changes: Vec<LockMigrationFileChange>,
    /// Complete authority-surface comparison between the two locks.
    pub migration: LockMigrationReport,
}

impl LockMigrationBridgeReport {
    /// Canonical pretty JSON artifact for human review and durable storage.
    pub fn json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self).map(|mut value| {
            value.push('\n');
            value
        })
    }

    /// Lowercase SHA-256 identity of every bound bridge field.
    /// Returns an empty identity if serialization cannot be produced.
    pub fn sha256(&self) -> String {
        let Ok(bytes) = serde_json::to_vec(self) else {
            return String::new();
        };
        crate::sha256_hex(&bytes)
    }
}

#[cfg(test)]
#[path = "bridge_test.rs"]
mod bridge_test;
