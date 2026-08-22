//! Shared construction of located Rust source facts.

use proc_macro2::Span;
use zrail_core::{AnalysisQuality, SourceSpan};

use super::{ObservedFact, SyntaxGuard};

pub(super) fn fact(name: impl Into<String>, span: Span, quality: AnalysisQuality) -> ObservedFact {
    ObservedFact {
        name: name.into(),
        canonical: Vec::new(),
        span: Some(source_span(span)),
        quality,
        guard: SyntaxGuard::Ordinary,
    }
}

pub(super) fn source_span(span: Span) -> SourceSpan {
    let start = span.start();
    let end = span.end();
    SourceSpan {
        line: start.line,
        column: start.column + 1,
        end_line: end.line,
        end_column: end.column + 1,
    }
}
