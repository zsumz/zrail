//! Missing and unresolved source relationships fail closed with stable diagnostics.

use zrail_core::{AnalysisQuality, Finding, SourceSpan};

use crate::source::ResolutionError;

use super::Walker;

impl Walker<'_> {
    pub(super) fn resolution_error(
        &mut self,
        origin: &str,
        span: Option<SourceSpan>,
        error: &ResolutionError,
        label: &str,
    ) {
        let message = format!("{label} cannot be resolved: {}", error.message());
        match error {
            ResolutionError::Escape(_) => self.boundary(origin, span, message),
            ResolutionError::Unresolved(_) => self.unresolved(origin, span, message),
        }
    }

    pub(super) fn missing(&mut self, origin: &str, span: Option<SourceSpan>, message: String) {
        if !self.reported.insert((origin.into(), message.clone())) {
            return;
        }
        self.findings.push(
            Finding::error(
                "RUST-GRAPH-001",
                "rust.source-graph.presence",
                "source-graph",
                message,
            )
            .at(origin, span),
        );
    }

    pub(super) fn unresolved(&mut self, origin: &str, span: Option<SourceSpan>, message: String) {
        if !self.reported.insert((origin.into(), message.clone())) {
            return;
        }
        self.findings.push(
            Finding::error(
                "RUST-GRAPH-003",
                "rust.source-graph.analysis",
                "source-graph",
                message,
            )
            .at(origin, span)
            .with_analysis(AnalysisQuality::Unresolved)
            .with_help("replace the boundary with a literal repository-local .rs source path"),
        );
    }
}
