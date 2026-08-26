//! Runtime-neutral async syntax is recorded separately from runtime effects.

use proc_macro2::Span;
use zrail_core::{AnalysisQuality, AsyncSyntax};

use super::{FactVisitor, fact::fact};

impl FactVisitor<'_> {
    pub(in crate::source) fn record_async_syntax(&mut self, kind: AsyncSyntax, span: Span) {
        let mut observation = fact(async_syntax_name(kind), span, AnalysisQuality::Exact);
        observation.lexical_scope.clone_from(&self.lexical_scope);
        self.async_syntax
            .push(super::AsyncSyntaxFact { kind, observation });
    }
}

const fn async_syntax_name(kind: AsyncSyntax) -> &'static str {
    match kind {
        AsyncSyntax::AsyncFn => "async fn",
        AsyncSyntax::AsyncBlock => "async block",
        AsyncSyntax::AsyncClosure => "async closure",
        AsyncSyntax::Await => "await",
    }
}
