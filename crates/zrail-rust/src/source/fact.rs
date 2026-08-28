//! Shared construction of located Rust source facts.

use proc_macro2::Span;
use zrail_core::{AnalysisQuality, SourceSpan};

use super::{ObservedFact, SyntaxGuard};

pub(super) fn written_path(path: &syn::Path) -> String {
    let mut written = path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>()
        .join("::");
    if path.leading_colon.is_some() {
        written.insert_str(0, "::");
    }
    written
}

pub(super) fn fact(name: impl Into<String>, span: Span, quality: AnalysisQuality) -> ObservedFact {
    ObservedFact {
        name: name.into(),
        written: None,
        implicit_prelude: super::ImplicitPreludeEligibility::Disabled,
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
    fact.implicit_prelude = super::ImplicitPreludeEligibility::Eligible;
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
