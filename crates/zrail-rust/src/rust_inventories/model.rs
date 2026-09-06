//! Complete quantities and bounded location samples describe only the declared syntax claim.

use serde::{Deserialize, Serialize};
use zrail_core::{AnalysisQuality, RustInventoryCount, RustInventoryRule, SourceSpan};

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
/// Content identity of one completely inspected physical Rust file.
pub struct RustInventoryInput {
    /// Normalized repository-relative file path.
    pub path: String,
    /// SHA-256 of all parsed UTF-8 source bytes.
    pub sha256: String,
    /// Complete source byte count.
    pub bytes: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
/// One authored syntax location; receiver identity is deliberately not claimed.
pub struct RustInventoryOccurrence {
    /// Physical ownership location.
    pub path: String,
    /// Written subject spelling.
    pub name: String,
    /// Exact syntax span: method identifier, complete path, import rename, or complete trait impl.
    pub span: SourceSpan,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
/// Auditable inventory including satisfied zero-count prohibitions.
pub struct GovernedRustInventory {
    /// Canonical `rust:inventory:<name>` policy identity.
    pub policy_id: String,
    /// Full selected scope, quantity, world, and justification.
    pub policy: RustInventoryRule,
    /// Explicit syntax claim, independent of receiver identity or execution.
    pub claim: String,
    /// Exact for the specified authored syntax; incomplete observations fail closed.
    pub quality: AnalysisQuality,
    /// All selected physical inputs, including files with no matching occurrences.
    pub inputs: Vec<RustInventoryInput>,
    /// Complete per-file/per-subject quantities in canonical order.
    pub counts: Vec<RustInventoryCount>,
    /// Complete selected-scope quantity before display truncation.
    pub observed_count: usize,
    /// At most sixteen source locations in canonical order.
    pub occurrence_sample: Vec<RustInventoryOccurrence>,
    /// Locations omitted from display only.
    pub occurrences_omitted: usize,
    /// Whether the complete observations satisfy the assertion.
    pub satisfied: bool,
}

#[derive(Debug, Default)]
pub(crate) struct RustInventoryAnalysis {
    pub(crate) policies: Vec<GovernedRustInventory>,
    pub(crate) binding_sha256: Option<String>,
}
