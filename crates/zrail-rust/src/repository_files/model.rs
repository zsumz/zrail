//! Auditable physical-entry and raw-content observations, separate from Rust identity.

use serde::{Deserialize, Serialize};
use zrail_core::{AnalysisQuality, Finding, RepositoryDocumentValue, RepositoryFileRule};

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
    /// Whether inspected bytes are valid UTF-8, only when the predicate requires it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_utf8: Option<bool>,
    /// Authored-document selection, only for structural document predicates.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub document: Option<RepositoryDocumentObservation>,
    /// First-marker offsets or complete line-value quantities for raw structure predicates.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_structure: Option<super::RepositoryTextStructureObservation>,
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

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// One exact literal-key selection; display omission never alters predicate evaluation.
pub struct RepositoryDocumentObservation {
    /// Exact authored type, or absent when the selected key does not exist.
    pub selected_type: Option<String>,
    /// Selected scalar/string array, displayed when its JSON encoding fits 16 KiB.
    pub value: Option<RepositoryDocumentValue>,
    /// True when the present value is oversized or outside the display value subset.
    /// The containing input hash always binds the complete document.
    pub value_omitted: bool,
    /// An intermediate key had a wrong structural type; this is never treated as absence.
    pub selection_error: Option<String>,
    /// Complete key-set quantities with bounded key samples, only for key predicates.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keys: Option<RepositoryDocumentKeys>,
    /// Immediate projected field type, when a field predicate observes an existing child.
    /// Parent absence/type remains independently visible in `selected_type` and `selection_error`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub field_type: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// Key quantities are complete even when individual names are omitted from display.
pub struct RepositoryDocumentKeys {
    /// Exact number of immediate keys, including an explicit non-table empty projection.
    pub count: usize,
    /// At most sixteen keys and 16 KiB of JSON-encoded key bytes, in lexical order.
    pub sample: Vec<String>,
    /// Keys omitted from the sample, never from comparison or the total.
    pub omitted: usize,
    /// Complete missing required keys for exact mode; empty for allowed mode.
    pub missing: Vec<String>,
    /// Complete number of keys outside the policy set.
    pub unexpected_count: usize,
    /// Bounded examples of unexpected keys, using the same sample limits.
    pub unexpected_sample: Vec<String>,
    /// Unexpected keys omitted from display.
    pub unexpected_omitted: usize,
}

#[derive(Debug, Default)]
pub(crate) struct RepositoryFileAnalysis {
    pub(crate) policies: Vec<GovernedRepositoryFile>,
    pub(crate) findings: Vec<Finding>,
    pub(crate) binding_sha256: Option<String>,
}
