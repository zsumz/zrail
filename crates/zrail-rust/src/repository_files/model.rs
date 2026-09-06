//! Auditable physical-entry and raw-content observations, separate from Rust identity.

use serde::{Deserialize, Serialize};
use zrail_core::{AnalysisQuality, Finding, RepositoryFileRule};

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// Complete evaluation of one repository-file policy, including empty selections.
pub struct GovernedRepositoryFile {
    /// Canonical `repository:file:<name>` policy identity.
    pub policy_id: String,
    /// Full authored selector, predicate, and justification.
    pub policy: RepositoryFileRule,
    /// Precisely bounded claim: physical paths, raw UTF-8 text, or file bytes.
    pub claim: String,
    /// Exact within the stated physical or raw-text claim; never execution evidence.
    pub analysis: AnalysisQuality,
    /// Actual canonical checkout prefix, only for filesystem-basis name policies.
    pub checkout_path: Option<String>,
    /// Complete distinct selected path inventory in lexical order, without truncation.
    pub entries: Vec<GovernedRepositoryFileEntry>,
    /// Additional byte-equality input, when present and a regular contained file.
    pub reference: Option<GovernedRepositoryFileEntry>,
    /// Whether the entire predicate, including cardinality and presence, holds.
    pub satisfied: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// One logical repository path; directory links are never recursively traversed.
pub struct GovernedRepositoryFileEntry {
    /// Written repository-relative selection path.
    pub path: String,
    /// Physical entry kind: file, directory, symlink, or other.
    pub kind: String,
    /// Contained canonical target when the written path resolves through a link.
    pub resolved_path: Option<String>,
    /// SHA-256 of bytes actually inspected; absent for path-only predicates.
    pub sha256: Option<String>,
    /// Exact byte length when contents were inspected.
    pub bytes: Option<usize>,
    /// Per-entry result; scope cardinality is reported on the containing policy.
    pub satisfied: bool,
    /// Total non-overlapping occurrences in the transformed UTF-8 text.
    pub literal_count: Option<usize>,
    /// Up to sixteen byte offsets in transformed text, not original-source spans.
    pub literal_offsets: Vec<usize>,
    /// Occurrences omitted from the displayed offset sample, never from the total.
    pub omitted_literal_offsets: usize,
    /// Exact disallowed written names observed in this path, in canonical order.
    pub forbidden_names: Vec<String>,
}

#[derive(Debug, Default)]
pub(crate) struct RepositoryFileAnalysis {
    pub(crate) policies: Vec<GovernedRepositoryFile>,
    pub(crate) findings: Vec<Finding>,
    pub(crate) binding_sha256: Option<String>,
}
