//! Scoped imports normalize alias chains and fail closed on cycles.

use std::fmt::Write as _;

use zrail_core::AnalysisQuality;

use super::{ScopedAlias, collect};

#[test]
fn aliases_resolve_against_their_scope_and_outer_imports() {
    let file = syn::parse_file("use runtime as rt; use rt::select as choose;")
        .expect("parse scoped imports");
    let aliases = collect(file.items.iter(), |name| {
        if name == "runtime" {
            external("tokio")
        } else {
            external(name)
        }
    });

    assert_eq!(aliases["rt"].target, "tokio");
    assert_eq!(aliases["choose"].target, "tokio::select");
}

#[test]
fn alias_cycles_are_unresolved() {
    let file = syn::parse_file("use b as a; use a as b;").expect("parse cyclic imports");
    let aliases = collect(file.items.iter(), external);

    assert_eq!(aliases["a"].quality, AnalysisQuality::Unresolved);
    assert_eq!(aliases["b"].quality, AnalysisQuality::Unresolved);
}

#[test]
fn conditional_aliases_never_create_exact_macro_authority() {
    let file = syn::parse_file("#[cfg(any())] use tokio as rt;").expect("parse conditional import");
    let aliases = collect(file.items.iter(), external);

    assert_eq!(aliases["rt"].target, "tokio");
    assert_eq!(aliases["rt"].quality, AnalysisQuality::Unresolved);
}

#[test]
fn local_modules_shadow_dependency_roots_in_their_lexical_scope() {
    let file = syn::parse_file("mod runtime {}").expect("parse local module");
    let aliases = collect(file.items.iter(), external);

    assert_eq!(aliases["runtime"].target, "runtime");
    assert_eq!(aliases["runtime"].quality, AnalysisQuality::Exact);
    assert!(aliases["runtime"].local_module);

    let conditional = syn::parse_file("#[cfg(unix)] mod runtime {}").expect("parse conditional");
    let conditional = collect(conditional.items.iter(), external);
    assert_eq!(conditional["runtime"].quality, AnalysisQuality::Unresolved);
}

#[test]
fn bare_local_macro_definitions_are_unresolved_only_in_their_scope() {
    let file = syn::parse_file("macro_rules! panic { () => {} }").expect("parse local macro");
    let aliases = collect(file.items.iter(), external);

    assert_eq!(aliases["panic"].target, "panic");
    assert_eq!(aliases["panic"].quality, AnalysisQuality::Unresolved);
    assert!(!aliases["panic"].local_module);
}

#[test]
fn expanded_alias_paths_have_a_fixed_byte_limit() {
    let path = std::iter::repeat_n("segment", 160)
        .collect::<Vec<_>>()
        .join("::");
    let file =
        syn::parse_file(&format!("use {path} as bounded;")).expect("parse oversized alias path");
    let aliases = collect(file.items.iter(), external);

    assert_eq!(aliases["bounded"].target, "bounded");
    assert_eq!(aliases["bounded"].quality, AnalysisQuality::Unresolved);
}

#[test]
fn alias_chains_have_a_fixed_depth_limit() {
    let source = (0..140).fold(String::new(), |mut source, index| {
        write!(source, "use a{} as a{index};", index + 1).expect("append alias");
        source
    });
    let file = syn::parse_file(&source).expect("parse deep alias chain");
    let aliases = collect(file.items.iter(), external);

    assert_eq!(aliases["a0"].quality, AnalysisQuality::Unresolved);
}

fn external(name: &str) -> ScopedAlias {
    ScopedAlias {
        target: name.into(),
        quality: AnalysisQuality::Exact,
        local_module: false,
    }
}
