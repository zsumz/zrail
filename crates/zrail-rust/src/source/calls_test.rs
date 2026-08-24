//! Call identities retain aliases and conservative glob-import possibilities.

use std::fmt::Write as _;

use zrail_core::AnalysisQuality;

use super::{ImportMap, SyntaxGuard, candidates, facts, macro_candidates, unresolved_projection};

#[test]
fn aliases_resolve_to_the_exact_called_path() {
    let file = syn::parse_file(
        "use std::process::Command as Process; fn run() { Process::new(\"git\"); }",
    )
    .expect("parse source");
    let imports = ImportMap::from_file(&file);

    let observed = facts(call(&file), &imports, SyntaxGuard::Ordinary, &[], &[]);

    assert!(observed.iter().any(|fact| {
        fact.name == "std::process::Command::new" && fact.quality == AnalysisQuality::Exact
    }));
}

#[test]
fn inherent_qualified_self_calls_retain_the_self_type() {
    let file = syn::parse_file("fn run() { <std::process::Command>::new(\"git\"); }")
        .expect("parse source");
    let imports = ImportMap::from_file(&file);

    let observed = facts(call(&file), &imports, SyntaxGuard::Ordinary, &[], &[]);

    assert!(observed.iter().any(|fact| {
        fact.name == "std::process::Command::new" && fact.quality == AnalysisQuality::Exact
    }));
}

#[test]
fn qualified_self_calls_resolve_imported_type_aliases() {
    let file =
        syn::parse_file("use std::process::Command as Spawn; fn run() { <Spawn>::new(\"git\"); }")
            .expect("parse source");
    let imports = ImportMap::from_file(&file);

    let observed = facts(call(&file), &imports, SyntaxGuard::Ordinary, &[], &[]);

    assert!(observed.iter().any(|fact| {
        fact.name == "std::process::Command::new" && fact.quality == AnalysisQuality::Exact
    }));
}

#[test]
fn generic_associated_self_paths_are_unresolved() {
    let file = syn::parse_file(
        "trait Factory { type Output; } fn run<process: Factory>() { <process::Output>::new(\"git\"); }",
    )
    .expect("parse source");
    let imports = ImportMap::from_file(&file);
    let generic_types = vec!["process".to_owned()];

    let observed = facts(
        call(&file),
        &imports,
        SyntaxGuard::Ordinary,
        &generic_types,
        &[],
    );

    assert!(observed.iter().any(|fact| {
        fact.name == "process::Output::new" && fact.quality == AnalysisQuality::Unresolved
    }));
}

#[test]
fn trait_qualified_calls_retain_the_named_trait_path() {
    let file = syn::parse_file(
        "trait Launch { fn launch(); } struct Type; fn run() { <Type as Launch>::launch(); }",
    )
    .expect("parse source");
    let imports = ImportMap::from_file(&file);

    let observed = facts(call(&file), &imports, SyntaxGuard::Ordinary, &[], &[]);

    assert!(
        observed.iter().any(|fact| {
            fact.name == "Launch::launch" && fact.quality == AnalysisQuality::Exact
        })
    );
}

#[test]
fn associated_type_qualified_calls_become_resolution_boundaries() {
    let file = syn::parse_file(
        "trait Provider { type Command; } struct Runtime; fn run() { <Runtime as Provider>::Command::new(\"git\"); }",
    )
    .expect("parse source");
    let imports = ImportMap::from_file(&file);
    let call = call(&file);

    let boundary = unresolved_projection(call, SyntaxGuard::Ordinary)
        .expect("associated type projection is unresolved");
    let observed = facts(call, &imports, SyntaxGuard::Ordinary, &[], &[]);

    assert_eq!(boundary.written, "<Runtime as Provider>::Command::new");
    assert!(observed.is_empty());
}

#[test]
fn non_path_qualified_self_calls_are_never_exact() {
    let file = syn::parse_file("fn run() { <(std::process::Command)>::new(\"git\"); }")
        .expect("parse source");
    let imports = ImportMap::from_file(&file);

    let observed = facts(call(&file), &imports, SyntaxGuard::Ordinary, &[], &[]);

    assert!(
        observed
            .iter()
            .any(|fact| { fact.name == "::new" && fact.quality == AnalysisQuality::Unresolved }),
        "{observed:#?}"
    );
}

#[test]
fn glob_imports_add_a_conservative_called_path() {
    let file = syn::parse_file("use std::process::*; fn run() { Command::new(\"git\"); }")
        .expect("parse source");
    let imports = ImportMap::from_file(&file);

    let observed = facts(call(&file), &imports, SyntaxGuard::Ordinary, &[], &[]);

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

    let observed = facts(call(&file), &imports, SyntaxGuard::Ordinary, &[], &[]);

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

    let observed = facts(call(&file), &imports, SyntaxGuard::Ordinary, &[], &[]);

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

#[test]
fn ordinary_macro_occurrences_include_test_only_alias_candidates() {
    let file = syn::parse_file("use serde::Serialize as Model; #[cfg(test)] use other::Model;")
        .expect("parse guarded macro aliases");
    let imports = ImportMap::from_file(&file);
    let path = syn::parse_str::<syn::Path>("Model").expect("parse macro path");

    let (candidates, overflowed) =
        macro_candidates(&path, &imports, "serde::Serialize", SyntaxGuard::Ordinary);

    assert!(!overflowed);
    assert!(
        candidates
            .iter()
            .any(|(candidate, _)| candidate.name == "other::Model")
    );
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
