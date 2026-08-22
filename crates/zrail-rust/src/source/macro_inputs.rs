//! Bounded traversal of ordinary Rust embedded in function-like macro inputs.

use proc_macro2::{TokenStream, TokenTree};
use syn::{Expr, Macro, Pat, Token, parse::Parse, parse::ParseStream, parse::Parser, visit::Visit};
use zrail_core::AnalysisQuality;

use super::{fact::fact, visitor::FactVisitor};

const MAX_SCANNED_TOKENS: usize = 8_192;

pub(super) fn inspect(visitor: &mut FactVisitor<'_>, invocation: &Macro, name: &str) -> bool {
    if invocation.tokens.is_empty() || directly_understood(name) {
        return false;
    }
    if !within_limit(invocation.tokens.clone()) {
        scan_tokens(visitor, invocation.tokens.clone());
        return true;
    }
    let leaf = name.rsplit("::").next().unwrap_or(name);
    let parsed = match leaf {
        "vec" => visit_vec(visitor, invocation.tokens.clone()),
        "matches" => visit_matches(visitor, invocation.tokens.clone()),
        "assert" | "assert_eq" | "assert_ne" | "concat" | "concat_bytes" | "debug_assert"
        | "debug_assert_eq" | "debug_assert_ne" | "eprint" | "eprintln" | "format"
        | "format_args" | "include" | "include_bytes" | "include_str" | "panic" | "print"
        | "println" | "write" | "writeln" => {
            visit_expression_list(visitor, invocation.tokens.clone())
        }
        _ => false,
    };
    if !parsed {
        scan_tokens(visitor, invocation.tokens.clone());
    }
    !parsed
}

fn within_limit(tokens: TokenStream) -> bool {
    let mut remaining = MAX_SCANNED_TOKENS;
    consume(tokens, &mut remaining)
}

fn consume(tokens: TokenStream, remaining: &mut usize) -> bool {
    for tree in tokens {
        let Some(next) = remaining.checked_sub(1) else {
            return false;
        };
        *remaining = next;
        if let TokenTree::Group(group) = tree
            && !consume(group.stream(), remaining)
        {
            return false;
        }
    }
    true
}

fn visit_expression_list(visitor: &mut FactVisitor<'_>, tokens: TokenStream) -> bool {
    let parser = syn::punctuated::Punctuated::<Expr, Token![,]>::parse_terminated;
    let Ok(expressions) = parser.parse2(tokens) else {
        return false;
    };
    for expression in &expressions {
        visitor.visit_expr(expression);
    }
    true
}

fn visit_vec(visitor: &mut FactVisitor<'_>, tokens: TokenStream) -> bool {
    if visit_expression_list(visitor, tokens.clone()) {
        return true;
    }
    let Ok(repeat) = syn::parse2::<VecRepeat>(tokens) else {
        return false;
    };
    visitor.visit_expr(&repeat.value);
    visitor.visit_expr(&repeat.count);
    true
}

fn visit_matches(visitor: &mut FactVisitor<'_>, tokens: TokenStream) -> bool {
    let Ok(input) = syn::parse2::<MatchesInput>(tokens) else {
        return false;
    };
    visitor.visit_expr(&input.expression);
    visitor.visit_pat(&input.pattern);
    if let Some(guard) = &input.guard {
        visitor.visit_expr(guard);
    }
    true
}

fn scan_tokens(visitor: &mut FactVisitor<'_>, tokens: TokenStream) {
    let mut budget = MAX_SCANNED_TOKENS;
    scan_stream(visitor, tokens, &mut budget);
}

fn scan_stream(visitor: &mut FactVisitor<'_>, tokens: TokenStream, budget: &mut usize) {
    if *budget == 0 {
        return;
    }
    let trees = tokens.into_iter().take(*budget).collect::<Vec<_>>();
    *budget = budget.saturating_sub(trees.len());
    let mut index = 0;
    while index < trees.len() {
        match &trees[index] {
            TokenTree::Group(group) => scan_stream(visitor, group.stream(), budget),
            TokenTree::Ident(identifier) if identifier == "unsafe" => {
                visitor.unsafe_constructs.push(fact(
                    "unsafe token in opaque macro input",
                    identifier.span(),
                    AnalysisQuality::Conservative,
                ));
            }
            TokenTree::Ident(_) => index = scan_path(visitor, &trees, index),
            _ => {}
        }
        index += 1;
    }
}

fn scan_path(visitor: &mut FactVisitor<'_>, trees: &[TokenTree], start: usize) -> usize {
    let TokenTree::Ident(first) = &trees[start] else {
        return start;
    };
    let mut segments = vec![first.to_string()];
    let mut end = start;
    while end + 3 < trees.len() && punct(&trees[end + 1], ':') && punct(&trees[end + 2], ':') {
        let TokenTree::Ident(identifier) = &trees[end + 3] else {
            break;
        };
        segments.push(identifier.to_string());
        end += 3;
    }
    let name = segments.join("::");
    visitor
        .paths
        .push(fact(&name, first.span(), AnalysisQuality::Conservative));
    if start > 0 && punct(&trees[start - 1], '.') {
        visitor.methods.push(fact(
            first.to_string(),
            first.span(),
            AnalysisQuality::Conservative,
        ));
    }
    if end + 1 < trees.len()
        && matches!(&trees[end + 1], TokenTree::Group(group) if group.delimiter() == proc_macro2::Delimiter::Parenthesis)
    {
        visitor
            .calls
            .push(fact(&name, first.span(), AnalysisQuality::Conservative));
    }
    if trees.get(end + 1).is_some_and(|tree| punct(tree, '!'))
        && let Some(TokenTree::Group(group)) = trees.get(end + 2)
        && let Ok(path) = syn::parse_str::<syn::Path>(&name)
    {
        let expansion = visitor.macro_invocation(&path);
        visitor.macros.extend(
            expansion
                .candidates
                .iter()
                .map(|candidate| candidate.observation.clone()),
        );
        super::compile_effects::record_tokens(visitor, group.stream(), &expansion, true);
        visitor.macro_expansions.push(expansion);
    }
    end
}

fn punct(tree: &TokenTree, character: char) -> bool {
    matches!(tree, TokenTree::Punct(punctuation) if punctuation.as_char() == character)
}

fn directly_understood(name: &str) -> bool {
    !name.contains("::")
        && matches!(
            name,
            "cfg" | "column" | "env" | "file" | "line" | "module_path" | "option_env" | "stringify"
        )
}

struct VecRepeat {
    value: Expr,
    count: Expr,
}

impl Parse for VecRepeat {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let value = input.parse()?;
        input.parse::<Token![;]>()?;
        let count = input.parse()?;
        let _ = input.parse::<Option<Token![,]>>()?;
        if !input.is_empty() {
            return Err(input.error("unexpected vec! input"));
        }
        Ok(Self { value, count })
    }
}

struct MatchesInput {
    expression: Expr,
    pattern: Pat,
    guard: Option<Expr>,
}

impl Parse for MatchesInput {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let expression = input.parse()?;
        input.parse::<Token![,]>()?;
        let pattern = Pat::parse_multi_with_leading_vert(input)?;
        let guard = if input.peek(Token![if]) {
            input.parse::<Token![if]>()?;
            Some(input.parse()?)
        } else {
            None
        };
        let _ = input.parse::<Option<Token![,]>>()?;
        if !input.is_empty() {
            return Err(input.error("unexpected matches! input"));
        }
        Ok(Self {
            expression,
            pattern,
            guard,
        })
    }
}

#[cfg(test)]
#[path = "macro_inputs_test.rs"]
mod macro_inputs_test;
