//! Source-owner policy coverage records.

use serde::Serialize;
use zrail_core::{AnalysisQuality, SourceSpan};

use super::GovernedCompilationDomain;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
/// One configured source owner and all matched source-operation occurrences.
pub struct GovernedOwnerRule {
    /// Canonical report identity for this policy.
    pub policy_id: String,
    /// Contract-authored rule name.
    pub name: String,
    /// Owner relationship kind in kebab-case.
    pub kind: String,
    /// Exact selector governed by this owner.
    pub target: String,
    /// Written receiver methods treated as mutation by this owner.
    pub mutating_methods: Vec<String>,
    /// Source reachability considered by this rule.
    pub reachability: String,
    /// Repository patterns limiting rule evaluation.
    pub within: Vec<String>,
    /// Exact package or source paths authorized to own the target.
    pub allow: Vec<String>,
    /// Contract-authored justification.
    pub reason: String,
    /// Matched operation occurrences in deterministic source order.
    pub occurrences: Vec<GovernedOperationOccurrence>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
/// One source operation selected by an enabled owner rule.
pub struct GovernedOperationOccurrence {
    /// Repository-relative source path.
    pub path: String,
    /// Operation relationship represented by this occurrence.
    pub operation: String,
    /// Analyzer-observed subject identity.
    pub observed: String,
    /// Written subject before canonical resolution, when retained.
    pub written: Option<String>,
    /// Written receiver method selected as mutation, when applicable.
    pub method: Option<String>,
    /// Every canonical identity retained by the analyzer.
    pub canonical: Vec<String>,
    /// Source coordinates, when the parser retained them.
    pub span: Option<SourceSpan>,
    /// Resolution confidence for the occurrence.
    pub quality: AnalysisQuality,
    /// Effective syntax guard in kebab-case.
    pub guard: String,
    /// Cargo compilation domains where the guarded occurrence is available.
    pub compilation_domains: Vec<GovernedCompilationDomain>,
    /// Whether the occurrence path is an exact configured owner.
    pub allowed: bool,
}
