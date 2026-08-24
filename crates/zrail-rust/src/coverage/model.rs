//! Public schema for governed-surface coverage reports.

use serde::Serialize;
use zrail_core::{AnalysisQuality, SourceSpan};

use crate::AnalysisMetrics;

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
    /// Governed-surface report schema; currently `1`.
    pub schema: u64,
    /// Schema of the fully merged architecture contract.
    pub contract_schema: u64,
    /// Complete analysis census and exclusion boundary.
    pub analysis: GovernedAnalysis,
    /// Matched owner occurrences whose identity was unresolved.
    pub unresolved_occurrences: usize,
    /// Matched owner occurrences conservatively mapped to multiple identities.
    pub ambiguous_occurrences: usize,
    /// Every enabled owner rule, ordered by canonical policy identity.
    pub owners: Vec<GovernedOwnerRule>,
    /// Every dependency prohibition and its shortest resolved violation paths.
    pub dependencies: Vec<GovernedDependencyRule>,
    /// Every declared production-to-test mirror identity.
    pub test_mirrors: Vec<GovernedTestMirror>,
}

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

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
/// One exact Cargo target compilation domain.
pub struct GovernedCompilationDomain {
    /// Cargo package owning the target.
    pub package: String,
    /// Rust edition used by the target.
    pub edition: String,
    /// Cargo target name.
    pub target: String,
    /// Compilation mode in kebab-case.
    pub mode: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
/// One configured package dependency prohibition.
pub struct GovernedDependencyRule {
    /// Canonical report identity for this policy.
    pub policy_id: String,
    /// Contract-authored rule name.
    pub name: String,
    /// Exact workspace package selected as the path origin.
    pub from: String,
    /// Denied resolved package names in canonical order.
    pub deny: Vec<String>,
    /// Direct or transitive graph reachability.
    pub reachability: String,
    /// Effective first-edge dependency kinds in canonical order.
    pub kinds: Vec<String>,
    /// Contract-authored justification.
    pub reason: String,
    /// Shortest exact resolved paths reaching denied packages.
    pub paths: Vec<GovernedDependencyPath>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
/// One shortest prohibited path through exact Cargo.lock nodes.
pub struct GovernedDependencyPath {
    /// Kind of the first manifest edge entering the resolved path.
    pub kind: String,
    /// Denied package name reached by this path.
    pub denied: String,
    /// Ordered exact package identities from workspace root to prohibition.
    pub nodes: Vec<GovernedPackageIdentity>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
/// One immutable package identity parsed from Cargo.lock.
pub struct GovernedPackageIdentity {
    /// Cargo package name.
    pub name: String,
    /// Exact locked package version.
    pub version: String,
    /// Exact Cargo.lock source or repository-local path identity.
    pub source: String,
    /// Exact Cargo.lock checksum, when present.
    pub checksum: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
/// One exact production-to-test mirror declaration.
pub struct GovernedTestMirror {
    /// Canonical report identity for this mirror.
    pub policy_id: String,
    /// Production-reachable Rust source path.
    pub production: String,
    /// Cargo-test-reachable Rust source path.
    pub test: String,
    /// Exact test function identifier.
    pub test_name: String,
    /// Repository-relative execution receipt path.
    pub receipt: String,
    /// Contract-authored justification.
    pub reason: String,
}
