//! Shared construction of located Rust source facts.

use proc_macro2::Span;
use zrail_core::{AnalysisQuality, SourceSpan};

use super::{ObservedFact, SyntaxGuard};

pub(super) fn fact(name: impl Into<String>, span: Span, quality: AnalysisQuality) -> ObservedFact {
    ObservedFact {
        name: name.into(),
        written: None,
        canonical: Vec::new(),
        span: Some(source_span(span)),
        quality,
        guard: SyntaxGuard::Ordinary,
        lexical_scope: Vec::new(),
        namespace: super::FactNamespace::Unknown,
    }
}

pub(super) fn written_fact(
    name: impl Into<String>,
    written: impl Into<String>,
    span: Span,
    quality: AnalysisQuality,
    lexical_scope: &[SourceSpan],
) -> ObservedFact {
    let mut fact = fact(name, span, quality);
    fact.written = Some(written.into());
    fact.lexical_scope = lexical_scope.to_vec();
    fact
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
