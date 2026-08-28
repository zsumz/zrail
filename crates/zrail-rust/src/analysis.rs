//! Complete analysis is explicit and incomplete observations cannot become lock authority.

use serde::{Deserialize, Serialize};
use zrail_core::AnalysisQuality;

use crate::source::SourceIndex;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
/// Stable category explaining why repository analysis is incomplete.
pub enum AnalysisIssueKind {
    /// Multiplicative source contexts exhausted their input-derived allowance.
    DerivedContextLimit,
    /// Source traversal exceeded its maximum ancestry depth.
    DepthLimit,
    /// Source traversal encountered a module or include cycle.
    Cycle,
    /// Include-dependent name resolution exhausted its work allowance.
    ProjectionWorkLimit,
    /// Include-dependent projection exhausted its retained-fact allowance.
    ProjectedFactLimit,
    /// A physical source input could not be parsed or safely bounded.
    SourceInput,
    /// Another source relationship could not be resolved authoritatively.
    Unresolved,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// One exact, non-baselineable reason that analysis cannot produce a lock.
pub struct AnalysisIssue {
    /// Stable diagnostic identity emitted by the analyzer.
    pub id: String,
    /// Typed incompleteness category.
    pub kind: AnalysisIssueKind,
    /// Repository-relative source path when the issue is localized.
    pub path: Option<String>,
    /// Human-readable description retaining the exact observed cause.
    pub message: String,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// Deterministic work census for one repository analysis.
pub struct AnalysisMetrics {
    /// Physical Rust files parsed successfully.
    pub physical_rust_files: usize,
    /// Syntax-specific source facts collected before contextual projection.
    pub physical_facts: usize,
    /// Input-sized `(file, compilation-domain)` contexts.
    pub base_contexts: usize,
    /// Additional contexts introduced by multiplicative source traversal.
    pub derived_contexts: usize,
    /// Physical files connected to include-dependent binding resolution.
    pub projection_files: usize,
    /// Actual include-dependent resolution transitions performed.
    pub projection_work: usize,
    /// New facts retained from include-dependent projection.
    pub projected_facts: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(tag = "status", rename_all = "kebab-case", deny_unknown_fields)]
/// Complete or incomplete result of repository-wide static analysis.
pub enum AnalysisOutcome {
    /// Every authoritative input and relationship was analyzed successfully.
    Complete {
        /// Deterministic work census for the completed analysis.
        metrics: AnalysisMetrics,
    },
    /// One or more non-baselineable analysis failures prevented lock construction.
    Incomplete {
        /// Deterministic work census collected before analysis stopped.
        metrics: AnalysisMetrics,
        /// Exact reasons lock construction is forbidden.
        issues: Vec<AnalysisIssue>,
    },
}

impl AnalysisOutcome {
    pub(crate) fn from_source(source: &SourceIndex) -> Self {
        let metrics = AnalysisMetrics {
            physical_rust_files: source.physical_file_count(),
            physical_facts: source.files.iter().map(crate::source::fact_count).sum(),
            base_contexts: source.analysis_metrics.base_contexts,
            derived_contexts: source.analysis_metrics.derived_contexts,
            projection_files: source.analysis_metrics.projection_files,
            projection_work: source.analysis_metrics.projection_work,
            projected_facts: source.analysis_metrics.projected_facts,
        };
        let issues = source
            .findings
            .iter()
            .filter(|finding| finding.analysis == AnalysisQuality::Unresolved)
            .map(|finding| AnalysisIssue {
                id: finding.id.clone(),
                kind: issue_kind(&finding.id, &finding.message),
                path: finding.path.clone(),
                message: finding.message.clone(),
            })
            .collect::<Vec<_>>();
        if issues.is_empty() {
            Self::Complete { metrics }
        } else {
            Self::Incomplete { metrics, issues }
        }
    }

    /// Returns whether the repository analysis can construct authoritative lock state.
    pub const fn is_complete(&self) -> bool {
        matches!(self, Self::Complete { .. })
    }

    /// Returns the deterministic work census for this analysis.
    pub const fn metrics(&self) -> AnalysisMetrics {
        match self {
            Self::Complete { metrics } | Self::Incomplete { metrics, .. } => *metrics,
        }
    }

    /// Returns typed incompleteness issues, or an empty slice for complete analysis.
    pub fn issues(&self) -> &[AnalysisIssue] {
        match self {
            Self::Complete { .. } => &[],
            Self::Incomplete { issues, .. } => issues,
        }
    }
}

fn issue_kind(id: &str, message: &str) -> AnalysisIssueKind {
    match id {
        "RUST-CONTEXT-001" => AnalysisIssueKind::DerivedContextLimit,
        "RUST-CONTEXT-002" => AnalysisIssueKind::DepthLimit,
        "RUST-CONTEXT-003" => AnalysisIssueKind::Cycle,
        "RUST-PROJECTION-001" => AnalysisIssueKind::ProjectionWorkLimit,
        "RUST-PROJECTION-002" => AnalysisIssueKind::ProjectedFactLimit,
        "RUST-PARSE-001" | "RUST-PARSE-002" => AnalysisIssueKind::SourceInput,
        _ if message.contains("projection") => AnalysisIssueKind::Unresolved,
        _ => AnalysisIssueKind::Unresolved,
    }
}
