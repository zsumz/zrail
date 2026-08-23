//! Call identities retain aliases and conservative glob-import possibilities.

use std::fmt::Write as _;

use zrail_core::AnalysisQuality;

use super::{ImportMap, SyntaxGuard, candidates, facts, macro_candidates};

#[test]
fn aliases_resolve_to_the_exact_called_path() {
    let file = syn::parse_file(
        "use std::process::Command as Process; fn run() { Process::new(\"git\"); }",
    )
    .expect("parse source");
    let imports = ImportMap::from_file(&file);

    let observed = facts(call(&file), &imports, SyntaxGuard::Ordinary, &[]);

    assert!(observed.iter().any(|fact| {
        fact.name == "std::process::Command::new" && fact.quality == AnalysisQuality::Exact
    }));
}

#[test]
fn glob_imports_add_a_conservative_called_path() {
    let file = syn::parse_file("use std::process::*; fn run() { Command::new(\"git\"); }")
        .expect("parse source");
    let imports = ImportMap::from_file(&file);

    let observed = facts(call(&file), &imports, SyntaxGuard::Ordinary, &[]);

    assert!(observed.iter().any(|fact| {
        fact.name == "std::process::Command::new" && fact.quality == AnalysisQuality::Conservative
    }));
}

#[test]
fn function_local_imports_add_a_conservative_called_path() {
    let file = syn::parse_file(
        "fn run() { use std::process::Command as Process; Process::new(\"git\"); }",
    )
    .expect("parse source");
    let imports = ImportMap::from_file(&file);

    let observed = facts(call(&file), &imports, SyntaxGuard::Ordinary, &[]);

    assert!(observed.iter().any(|fact| {
        fact.name == "std::process::Command::new" && fact.quality == AnalysisQuality::Conservative
    }));
}

#[test]
fn type_aliases_add_a_conservative_called_path() {
    let file = syn::parse_file(
        "type Process = std::process::Command; fn run() { Process::new(\"git\"); }",
    )
    .expect("parse source");
    let imports = ImportMap::from_file(&file);

    let observed = facts(call(&file), &imports, SyntaxGuard::Ordinary, &[]);

    assert!(observed.iter().any(|fact| {
        fact.name == "std::process::Command::new" && fact.quality == AnalysisQuality::Conservative
    }));
}

#[test]
fn type_aliases_add_a_conservative_reference_path() {
    let file = syn::parse_file(
        "type Process = std::process::Command; fn run() { let constructor = Process::new; }",
    )
    .expect("parse source");
    let imports = ImportMap::from_file(&file);
    let path = syn::parse_str::<syn::Path>("Process::new").expect("parse path");

    let observed = candidates(&path, &imports, "Process::new", SyntaxGuard::Ordinary);

    assert!(observed.iter().any(|fact| {
        fact.name == "std::process::Command::new" && fact.quality == AnalysisQuality::Conservative
    }));
}

#[test]
fn macro_candidate_sets_fail_closed_at_the_fixed_limit() {
    let mut imports = String::new();
    for index in 0..=super::MAX_MACRO_CANDIDATES {
        writeln!(imports, "use module_{index}::*;").expect("append macro import");
    }
    let file = syn::parse_file(&imports).expect("parse bounded macro imports");
    let imports = ImportMap::from_file(&file);
    let path = syn::parse_str::<syn::Path>("reviewed").expect("parse macro path");

    let (candidates, overflowed) =
        macro_candidates(&path, &imports, "reviewed", SyntaxGuard::Ordinary);

    assert!(overflowed);
    assert!(candidates.is_empty());
}

#[test]
fn inline_module_globs_do_not_become_file_wide_macro_candidates() {
    let file = syn::parse_file(
        "use dependency::*; mod tests { use super::*; fn run() { assert!(true); } }",
    )
    .expect("parse nested glob imports");
    let imports = ImportMap::from_file(&file);
    let path = syn::parse_str::<syn::Path>("assert").expect("parse macro path");

    let (candidates, overflowed) =
        macro_candidates(&path, &imports, "assert", SyntaxGuard::Ordinary);

    assert!(!overflowed);
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].0.name, "dependency::assert");
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
            syn::Stmt::Expr(syn::Expr::Call(call), _) => Some(call),
            _ => None,
        })
        .expect("function has a call")
}
