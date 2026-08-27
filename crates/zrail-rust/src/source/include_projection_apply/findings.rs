//! Projection failures retain exact typed incompleteness and bounded diagnostics.

use zrail_core::{AnalysisQuality, Finding};

use super::super::{SourceInstanceIssue, include_projection_budget::ProjectionLimit};

pub(in crate::source) fn unresolved(
    path: Option<&str>,
    span: Option<zrail_core::SourceSpan>,
) -> Finding {
    let mut finding = Finding::error(
        "RUST-INCLUDE-002",
        "rust.source.include-bindings",
        "source",
        "ordinary Rust path bindings could not be resolved completely",
    );
    if let Some(path) = path {
        finding = finding.at(path, span);
    }
    finding
        .with_analysis(AnalysisQuality::Unresolved)
        .with_help("reduce include or import indirection before trusting path and call authority")
}

pub(super) fn context_issue(issue: &SourceInstanceIssue) -> Finding {
    let (id, message, path) = match issue {
        SourceInstanceIssue::DerivedContextLimit { used, limit, file } => (
            "RUST-CONTEXT-001",
            format!(
                "derived Rust source contexts reached {used}, exceeding the input-derived limit {limit}"
            ),
            Some(file.as_str()),
        ),
        SourceInstanceIssue::DepthLimit { file, depth, chain } => (
            "RUST-CONTEXT-002",
            format!(
                "Rust source context depth reached {depth} through {}",
                chain.join(" -> ")
            ),
            Some(file.as_str()),
        ),
        SourceInstanceIssue::Cycle { chain } => (
            "RUST-CONTEXT-003",
            format!("Rust source context cycle: {}", chain.join(" -> ")),
            chain.last().map(String::as_str),
        ),
    };
    let finding = Finding::error(id, "rust.source.contexts", "source", message)
        .with_analysis(AnalysisQuality::Unresolved)
        .with_help("remove the pathological source expansion before constructing lock authority");
    path.map_or(finding.clone(), |path| finding.at(path, None))
}

pub(in crate::source) fn budget_exhausted(limit: ProjectionLimit) -> Finding {
    let (id, exhausted) = match limit {
        ProjectionLimit::Work => ("RUST-PROJECTION-001", "work"),
        ProjectionLimit::Facts => ("RUST-PROJECTION-002", "fact"),
    };
    Finding::error(
        id,
        "rust.source.include-bindings",
        "source",
        format!("repository-wide Rust binding projection exhausted its {exhausted} safety budget"),
    )
    .with_analysis(AnalysisQuality::Unresolved)
    .with_help(
        "reduce namespace occurrences or binding indirection before trusting source authority",
    )
}
