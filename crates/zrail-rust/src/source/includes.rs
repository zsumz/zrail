//! Include macro facts distinguish literal local source from unresolved generation.

use syn::{Expr, Macro, Token, parse::Parser, punctuated::Punctuated, spanned::Spanned};
use zrail_core::SourceSpan;

use super::{
    fact::source_span,
    model::{IncludeBoundary, IncludeContext},
};

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
    pub(crate) domain: super::CompilationDomain,
    pub(crate) guard: super::SyntaxGuard,
    pub(crate) context: IncludeContext,
    pub(crate) parent_scope: Vec<SourceSpan>,
    pub(crate) generic_types: Vec<String>,
    pub(crate) prelude_value_shadows: Vec<(String, super::SyntaxGuard)>,
    pub(crate) include_span: SourceSpan,
    pub(crate) occurrence: IncludeOccurrenceId,
}

pub(super) fn include_boundary(
    invocation: &Macro,
    context: IncludeContext,
) -> Option<IncludeBoundary> {
    if !invocation.path.is_ident("include") {
        return None;
    }
    let expression = invocation.tokens.to_string();
    let out_dir = out_dir_path(invocation);
    let span = source_span(invocation.span());
    Some(IncludeBoundary {
        path: literal_include_path(invocation),
        generated: out_dir.is_some() || expression.contains("OUT_DIR"),
        out_dir,
        expression,
        guard: super::SyntaxGuard::Ordinary,
        context,
        lexical_scope: Vec::new(),
        generic_types: Vec::new(),
        prelude_value_shadows: Vec::new(),
        occurrence: IncludeOccurrenceId::new(span),
        span: Some(span),
    })
}

fn out_dir_path(invocation: &Macro) -> Option<String> {
    let argument = only_argument(invocation)?;
    let Expr::Macro(concat) = argument else {
        return None;
    };
    if !concat.mac.path.is_ident("concat") {
        return None;
    }
    let arguments = parse_arguments(&concat.mac)?;
    let [Expr::Macro(environment), Expr::Lit(suffix)] = arguments.as_slice() else {
        return None;
    };
    if !environment.mac.path.is_ident("env") || env_name(&environment.mac)? != "OUT_DIR" {
        return None;
    }
    let syn::Lit::Str(suffix) = &suffix.lit else {
        return None;
    };
    normalized_output(suffix.value().strip_prefix('/')?)
}

fn only_argument(invocation: &Macro) -> Option<Expr> {
    let arguments = parse_arguments(invocation)?;
    if arguments.len() == 1 {
        arguments.into_iter().next()
    } else {
        None
    }
}

fn parse_arguments(invocation: &Macro) -> Option<Vec<Expr>> {
    let parser = Punctuated::<Expr, Token![,]>::parse_terminated;
    parser
        .parse2(invocation.tokens.clone())
        .ok()
        .map(|arguments| arguments.into_iter().collect())
}

fn env_name(invocation: &Macro) -> Option<String> {
    syn::parse2::<syn::LitStr>(invocation.tokens.clone())
        .ok()
        .map(|value| value.value())
}

fn normalized_output(output: &str) -> Option<String> {
    if output.is_empty()
        || output.contains('\\')
        || output
            .split('/')
            .any(|component| component.is_empty() || matches!(component, "." | ".."))
    {
        None
    } else {
        Some(output.to_owned())
    }
}

fn literal_include_path(invocation: &Macro) -> Option<String> {
    let Expr::Lit(literal) = only_argument(invocation)? else {
        return None;
    };
    let syn::Lit::Str(path) = &literal.lit else {
        return None;
    };
    Some(path.value())
}

#[cfg(test)]
#[path = "includes_test.rs"]
mod includes_test;
