//! Canonical lock identity for one execution-backed test mirror.

use serde::{Deserialize, Serialize};

/// Exact mirror identity and receipt bytes reviewed in `zrail.lock`.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LockedExecutionReceipt {
    /// Production source attested by the mirror.
    pub production: String,
    /// Cargo-test-reachable source declaring the exact test.
    pub test: String,
    /// Exact named test recorded by the receipt.
    pub name: String,
    /// Repository-relative receipt path.
    pub receipt: String,
    /// Lowercase SHA-256 digest of the exact receipt bytes.
    pub sha256: String,
    /// Input digest declared by the receipt producer.
    pub input_sha256: String,
    /// Versioned receipt-producer identity.
    pub producer: String,
}

#[cfg(test)]
#[path = "receipt_test.rs"]
mod receipt_test;
