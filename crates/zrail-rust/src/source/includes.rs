//! Include macro facts distinguish literal local source from unresolved generation.

use syn::{Expr, Macro, Token, parse::Parser, punctuated::Punctuated, spanned::Spanned};

use super::{
    fact::source_span,
    model::{IncludeBoundary, IncludeContext},
};

pub(super) fn include_boundary(
    invocation: &Macro,
    context: IncludeContext,
) -> Option<IncludeBoundary> {
    if !invocation.path.is_ident("include") {
        return None;
    }
    let expression = invocation.tokens.to_string();
    let out_dir = out_dir_path(invocation);
    Some(IncludeBoundary {
        path: literal_include_path(invocation),
        generated: out_dir.is_some() || expression.contains("OUT_DIR"),
        out_dir,
        expression,
        context,
        span: Some(source_span(invocation.span())),
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
