//! Whole-lock evidence separates full counts and differences from bounded display samples.

use serde::{Deserialize, Serialize};
use zrail_core::{AnalysisQuality, LockPackageIdentity, LockPackageRule};

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// One complete inventory over all Cargo.lock nodes with the selected package name.
pub struct GovernedLockPackage {
    /// Canonical policy identity used by diagnostics, coverage, and explanation.
    pub policy_id: String,
    /// Effective exact selector, assertion, and human justification.
    pub policy: LockPackageRule,
    /// Always `whole-cargo-lock`, including nodes unreachable from current manifests.
    pub scope: String,
    /// Always exact; missing or incomplete Cargo resolution cannot produce this report.
    pub quality: AnalysisQuality,
    /// SHA-256 of the complete parsed Cargo.lock bytes, also bound into candidate locks.
    pub lock_sha256: String,
    /// Complete number of parsed nodes before package-name selection.
    pub lock_node_count: usize,
    /// Complete number of matching nodes; dependency edges do not multiply occurrences.
    pub observed_count: usize,
    /// Canonically ordered display sample, limited to 16 identities and 16384 JSON bytes.
    pub observed_sample: Vec<LockPackageIdentity>,
    /// Matching nodes omitted from the display sample, still included in comparison.
    pub observed_omitted: usize,
    /// Complete missing set and bounded unexpected-node sample for exact mode only.
    pub exact_difference: Option<LockPackageDifference>,
    /// Result of comparing all selected identities or the complete selected count.
    pub satisfied: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// Identity-level differences; a same-count substitution is still a violation.
pub struct LockPackageDifference {
    /// Every expected identity absent from the selected nodes.
    pub missing: Vec<LockPackageIdentity>,
    /// Complete number of unexpected identities, including omitted samples.
    pub unexpected_count: usize,
    /// Canonical unexpected-node sample, limited to 16 identities and 16384 JSON bytes.
    pub unexpected_sample: Vec<LockPackageIdentity>,
    /// Unexpected identities omitted from display, still counted as violations.
    pub unexpected_omitted: usize,
}
