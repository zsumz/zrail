//! Runtime-neutral syntax policy has diagnostics distinct from runtime effects.

use zrail_core::{AsyncSyntax, Finding};

use crate::source::{AsyncSyntaxFact, MacroExpansionFact, RustFileFacts};

pub(super) fn finding(file: &RustFileFacts, fact: &AsyncSyntaxFact, profile: &str) -> Finding {
    Finding::error(
        "SYNTAX-001",
        format!("profile.{profile}.syntax"),
        "syntax",
        format!(
            "profile {profile:?} denies {} syntax",
            syntax_name(fact.kind)
        ),
    )
    .at(&file.relative, fact.observation.span)
    .with_analysis(fact.observation.quality)
    .with_help("move the async boundary to an outer adapter and expose a synchronous interface")
}

pub(super) fn opaque_macro(
    file: &RustFileFacts,
    expansion: &MacroExpansionFact,
    profile: &str,
) -> Finding {
    Finding::error(
        "SYNTAX-002",
        format!("profile.{profile}.syntax"),
        "syntax",
        format!(
            "profile {profile:?} cannot prove that opaque macro {} introduces no denied async syntax",
            expansion.name
        ),
    )
    .at(&file.relative, expansion.span)
    .with_analysis(expansion.quality)
    .with_help(
        "remove the macro or add exact content-bound macro authority with async_syntax = \"none\"",
    )
}

pub(crate) const fn syntax_name(kind: AsyncSyntax) -> &'static str {
    match kind {
        AsyncSyntax::AsyncFn => "async-fn",
        AsyncSyntax::AsyncBlock => "async-block",
        AsyncSyntax::AsyncClosure => "async-closure",
        AsyncSyntax::Await => "await",
    }
}
