//! Physical line measurements expose thresholds, provenance, and active debt without truncation.

use std::collections::BTreeSet;

use serde::Serialize;
use zrail_core::{AnalysisQuality, RatchetContract};

use crate::{EffectiveSizeBudget, engine::RepositoryModel, source_budget};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
/// One physical source's effective size policy and measured debt.
pub struct GovernedSizeBudget {
    /// Exact repository-relative source path; repeated mounts are counted once.
    pub path: String,
    /// Package ownership established by the source graph.
    pub packages: Vec<String>,
    /// Written physical line count, using Rust `str::lines()` semantics.
    pub lines: usize,
    /// Selected global/scoped thresholds and exact hard exception.
    pub effective: EffectiveSizeBudget,
    /// Every line above the selected design target.
    pub above_target: usize,
    /// Every line above the optional advisory soft threshold.
    pub above_soft: usize,
    /// Every line above the original hard ceiling, including explicitly allowed debt.
    pub above_hard: usize,
    /// Every line exceeding the effective hard bound, including any exception.
    pub above_effective_hard: usize,
    /// Authored ratchet identity, justification, and optional exact baseline.
    pub ratchet: Option<RatchetContract>,
    /// Exact physical-text measurement; this is not execution evidence.
    pub quality: AnalysisQuality,
}

pub(super) fn report(model: &RepositoryModel) -> Result<Vec<GovernedSizeBudget>, String> {
    let mut seen = BTreeSet::new();
    let mut result = Vec::new();
    for file in &model.source.files {
        if !seen.insert(&file.relative) {
            continue;
        }
        let Some(effective) = source_budget::for_file(file, &model.bundle.contract.source.rust)?
        else {
            continue;
        };
        result.push(GovernedSizeBudget {
            path: file.relative.clone(),
            packages: file.packages.clone(),
            lines: file.lines,
            above_target: file.lines.saturating_sub(effective.thresholds.target),
            above_soft: effective
                .thresholds
                .soft
                .map_or(0, |soft| file.lines.saturating_sub(soft)),
            above_hard: file.lines.saturating_sub(effective.thresholds.hard),
            above_effective_hard: file.lines.saturating_sub(effective.hard_ceiling()),
            ratchet: model
                .bundle
                .contract
                .ratchets
                .iter()
                .find(|ratchet| ratchet.rule == "rust.file-size" && ratchet.target == file.relative)
                .cloned(),
            effective,
            quality: AnalysisQuality::Exact,
        });
    }
    result.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(result)
}
