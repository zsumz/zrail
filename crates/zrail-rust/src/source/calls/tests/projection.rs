//! Transparent call syntax cannot hide an unresolved associated-type projection.

use proc_macro2::{Delimiter, Group, TokenStream, TokenTree};
use syn::visit::Visit;
use zrail_core::AnalysisQuality;

use super::super::visitor::FactVisitor;
use super::{ImportMap, SyntaxGuard, callee_path, facts, unresolved_path_projection};

#[test]
fn parenthesized_associated_type_projection_is_a_boundary() {
    assert_projection_boundary(
        "(<Runtime as Provider>::Command::new)(\"git\")",
        "<Runtime as Provider>::Command::new",
    );
}

#[test]
fn multiply_parenthesized_associated_type_projection_is_a_boundary() {
    assert_projection_boundary(
        "(((<Runtime as Provider>::Command::new)))(\"git\")",
        "<Runtime as Provider>::Command::new",
    );
}

#[test]
fn grouped_associated_type_projection_is_a_boundary() {
    let expression = grouped_call();
    let syn::Expr::Call(call) = &expression else {
        panic!("expression is a call");
    };
    assert!(matches!(call.func.as_ref(), syn::Expr::Group(_)));

    let path = callee_path(call.func.as_ref()).expect("grouped callable path");
    let boundary = unresolved_path_projection(path, SyntaxGuard::Ordinary, &[])
        .expect("grouped associated projection is unresolved");

    assert_eq!(boundary.written, "<Runtime as Provider>::Command::new");
    assert!(
        facts(
            call,
            &ImportMap::default(),
            &SyntaxGuard::Ordinary,
            &[],
            &[]
        )
        .is_empty()
    );

    let imports = ImportMap::default();
    let mut visitor = FactVisitor::new(&imports);
    visitor.visit_expr(&expression);
    assert_eq!(visitor.call_resolutions.len(), 1);
}

#[test]
fn parenthesized_direct_trait_function_remains_exact() {
    let file = parse_file("(<Runtime as Launch>::launch)()");
    let call = call(&file);
    let observed = facts(
        call,
        &ImportMap::from_file(&file),
        &SyntaxGuard::Ordinary,
        &[],
        &[],
    );

    assert!(
        observed.iter().any(|fact| {
            fact.name == "Launch::launch" && fact.quality == AnalysisQuality::Exact
        })
    );
}

fn assert_projection_boundary(expression: &str, written: &str) {
    let file = parse_file(expression);
    let call = call(&file);
    let path = callee_path(call.func.as_ref()).expect("callable path");
    let boundary = unresolved_path_projection(path, SyntaxGuard::Ordinary, &[])
        .expect("associated projection is unresolved");

    assert_eq!(boundary.written, written);
    assert!(
        facts(
            call,
            &ImportMap::from_file(&file),
            &SyntaxGuard::Ordinary,
            &[],
            &[]
        )
        .is_empty()
    );
}

fn grouped_call() -> syn::Expr {
    let projection = "<Runtime as Provider>::Command::new"
        .parse::<TokenStream>()
        .expect("projection tokens");
    let arguments = "\"git\"".parse::<TokenStream>().expect("argument tokens");
    let tokens = [
        TokenTree::Group(Group::new(Delimiter::None, projection)),
        TokenTree::Group(Group::new(Delimiter::Parenthesis, arguments)),
    ]
    .into_iter()
    .collect();
    syn::parse2::<syn::Expr>(tokens).expect("grouped call")
}

fn parse_file(expression: &str) -> syn::File {
    syn::parse_file(&format!(
        "trait Provider {{ type Command; }} trait Launch {{ fn launch(); }} struct Runtime; fn run() {{ let _ = {expression}; }}"
    ))
    .expect("parse call fixture")
}

fn call(file: &syn::File) -> &syn::ExprCall {
    let Some(syn::Item::Fn(function)) = file.items.last() else {
        panic!("last item is a function");
    };
    function
        .block
        .stmts
        .iter()
        .find_map(|statement| match statement {
            syn::Stmt::Local(local) => match local.init.as_ref()?.expr.as_ref() {
                syn::Expr::Call(call) => Some(call),
                _ => None,
            },
            _ => None,
        })
        .expect("function has a call")
}
