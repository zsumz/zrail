//! Content-bound certificate for the complete analyzed repository universe.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// Complete input census and deterministic workload retained in `zrail.lock`.
pub struct LockedAnalysis {
    /// Canonical digest of active manifests, packages, targets, and Rust files.
    pub inventory_sha256: String,
    /// Canonical digest of normalized repository exclusion patterns.
    pub exclusions_sha256: String,
    /// Exact Cargo.lock bytes when resolved Cargo authority participates.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cargo_lock_sha256: Option<String>,
    /// Number of active Cargo packages.
    pub packages: usize,
    /// Number of active Cargo targets.
    pub targets: usize,
    /// Number of successfully parsed physical Rust files.
    pub physical_rust_files: usize,
    /// Number of input-sized source contexts.
    pub base_source_contexts: usize,
    /// Number of multiplicative derived source contexts.
    pub derived_source_contexts: usize,
    /// Number of physical source facts before contextual projection.
    pub source_facts: usize,
    /// Include-dependent resolution transitions performed.
    pub projection_queries: usize,
    /// Newly retained include-projected facts.
    pub projected_facts: usize,
    /// Unresolved completeness findings; current locks require zero.
    pub unresolved_bindings: usize,
    /// Analyzer interpretation used to produce this certificate.
    pub analyzer_semantics: u64,
    /// Exact contract fragments and root bytes participating in authority.
    #[serde(default, rename = "contract_source")]
    pub contract_sources: Vec<LockedContractSource>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// One exact contract source bound into the completeness certificate.
pub struct LockedContractSource {
    /// Normalized repository-relative contract path.
    pub path: String,
    /// Lowercase SHA-256 digest of the exact source bytes.
    pub sha256: String,
}

impl Default for LockedAnalysis {
    fn default() -> Self {
        Self {
            inventory_sha256: crate::sha256_hex(b""),
            exclusions_sha256: crate::sha256_hex(b""),
            cargo_lock_sha256: None,
            packages: 0,
            targets: 0,
            physical_rust_files: 0,
            base_source_contexts: 0,
            derived_source_contexts: 0,
            source_facts: 0,
            projection_queries: 0,
            projected_facts: 0,
            unresolved_bindings: 0,
            analyzer_semantics: super::LOCK_SEMANTICS,
            contract_sources: Vec::new(),
        }
    }
}

impl LockedAnalysis {
    /// Compares authoritative coverage identity while ignoring diagnostic counts.
    pub fn same_authority(&self, other: &Self) -> bool {
        self.inventory_sha256 == other.inventory_sha256
            && self.exclusions_sha256 == other.exclusions_sha256
            && self.cargo_lock_sha256 == other.cargo_lock_sha256
            && self.unresolved_bindings == other.unresolved_bindings
            && self.analyzer_semantics == other.analyzer_semantics
            && self.contract_sources == other.contract_sources
    }
}
