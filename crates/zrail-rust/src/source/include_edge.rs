//! Exact include edges retain lexical position and occurrence identity.

use zrail_core::SourceSpan;

use super::{CompilationDomain, IncludeContext, SyntaxGuard};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct IncludeOccurrenceId {
    span: SourceSpan,
}

impl IncludeOccurrenceId {
    pub(crate) const fn new(span: SourceSpan) -> Self {
        Self { span }
    }

    pub(crate) const fn span(self) -> SourceSpan {
        self.span
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct CompilationIncludeEdge {
    pub(crate) parent: String,
    pub(crate) child: String,
    pub(crate) domain: CompilationDomain,
    pub(crate) guard: SyntaxGuard,
    pub(crate) context: IncludeContext,
    pub(crate) parent_scope: Vec<SourceSpan>,
    pub(crate) include_span: SourceSpan,
    pub(crate) occurrence: IncludeOccurrenceId,
}
