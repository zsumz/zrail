//! Typed, deterministic size-parity evidence, explicitly separate from source certificates.

use serde::Serialize;
use zrail_core::RatchetContract;

use crate::source_budget::EffectiveSizeBudget;

#[derive(Serialize)]
pub(super) struct Report {
    pub(super) schema: u32,
    pub(super) status: &'static str,
    pub(super) full_repository_qualified: bool,
    pub(super) claim: &'static str,
    pub(super) implementation_commit: String,
    pub(super) tracked_diff_sha256: String,
    pub(super) reports: Vec<Surface>,
    pub(super) limitations: [&'static str; 4],
}

#[derive(Serialize)]
pub(super) struct Surface {
    pub(super) snapshot: serde_json::Value,
    pub(super) policy_sha256: String,
    pub(super) legacy_policy_sha256: String,
    pub(super) physical_files: usize,
    pub(super) baseline_instances: usize,
    pub(super) hard_allow_instances: usize,
    pub(super) files: Vec<SourceRecord>,
}

#[derive(Serialize)]
pub(super) struct SourceRecord {
    pub(super) path: String,
    pub(super) source_sha256: String,
    pub(super) policy: EffectiveSizeBudget,
    pub(super) baseline: Option<RatchetContract>,
    pub(super) outcomes: Vec<Outcome>,
}

#[derive(Serialize)]
pub(super) struct Outcome {
    pub(super) case: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) lines: Option<usize>,
    pub(super) legacy_errors: Vec<String>,
    pub(super) native_error_ids: Vec<String>,
}
