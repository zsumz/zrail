//! Top-level governed-surface coverage records.

use serde::Serialize;

use crate::AnalysisMetrics;

use super::{
    GovernedDependencyRule, GovernedFeatureWorld, GovernedOwnerRule, GovernedSourcePolicyRail,
    GovernedTestMirror,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
/// Complete static-analysis census and exact repository exclusions.
pub struct GovernedAnalysis {
    /// Always `true`; incomplete analysis fails before a report is constructed.
    pub complete: bool,
    /// Deterministic input and work metrics from complete analysis.
    pub metrics: AnalysisMetrics,
    /// Normalized configured repository exclusions in canonical order.
    pub exclusions: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
/// Stable audit report for all configured owner and dependency prohibitions.
pub struct GovernedSurfaceReport {
    /// Governed-surface report schema; currently `5`.
    pub schema: u64,
    /// Schema of the fully merged architecture contract.
    pub contract_schema: u64,
    /// SHA-256 of the exact fully resolved contract bundle.
    pub contract_sha256: String,
    /// Complete analysis census and exclusion boundary.
    pub analysis: GovernedAnalysis,
    /// Matched owner occurrences whose identity was unresolved.
    pub unresolved_occurrences: usize,
    /// Matched owner occurrences conservatively mapped to multiple identities.
    pub ambiguous_occurrences: usize,
    /// Canonical identity of every enabled global or named policy rail.
    pub enabled_rails: Vec<String>,
    /// Every workspace-wide Cargo feature world accepted by the conservative proof boundary.
    pub feature_worlds: Vec<GovernedFeatureWorld>,
    /// Runtime-neutral syntax and written-import policies with exact occurrences.
    pub source_policies: Vec<GovernedSourcePolicyRail>,
    /// Every exact Rust type policy and its declaration and duplication observations.
    pub type_policies: Vec<super::super::GovernedTypePolicy>,
    /// Every enabled owner rule, ordered by canonical policy identity.
    pub owners: Vec<GovernedOwnerRule>,
    /// Every dependency prohibition and its shortest resolved violation paths.
    pub dependencies: Vec<GovernedDependencyRule>,
    /// Every declared production-to-test mirror identity.
    pub test_mirrors: Vec<GovernedTestMirror>,
}
