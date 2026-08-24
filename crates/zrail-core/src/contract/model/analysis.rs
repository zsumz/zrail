//! Reviewed analysis-expansion limits stay in the content-bound contract.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// Optional deterministic overrides for input-derived analyzer budgets.
pub struct AnalysisContract {
    /// Expansion limits applied while producing complete source facts.
    #[serde(default)]
    pub limits: AnalysisLimits,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// Explicit overrides for multiplicative analysis work, never ordinary input size.
pub struct AnalysisLimits {
    /// Maximum additional source contexts created by repeated ancestry paths.
    pub derived_source_instances: Option<usize>,
    /// Maximum include-binding resolution transitions.
    pub include_projection_work: Option<usize>,
    /// Maximum newly retained include-projected facts.
    pub projected_facts: Option<usize>,
}
