//! Alias, alias-chain, and grouped-import resolution examples.

use std::fmt::Write as _;

use zrail_core::AnalysisQuality;

use super::ImportMap;

#[test]
fn renamed_imports_resolve_to_their_declared_capability() {
    let file =
        syn::parse_file("use std::{net::TcpStream as Hidden, fs::*};").expect("parse imports");
    let imports = ImportMap::from_file(&file);
    let path = syn::parse_str::<syn::Path>("Hidden::connect").expect("parse path");
    let (resolved, quality) = imports.resolve(&path);

    assert_eq!(resolved, "std::net::TcpStream::connect");
    assert_eq!(quality, AnalysisQuality::Exact);
    assert_eq!(
        imports.declared_paths(),
        [("std::net::TcpStream", AnalysisQuality::Exact)]
    );
    assert_eq!(imports.globs(), &["std::fs"]);
}

#[test]
fn alias_chains_resolve_to_the_original_crate() {
    let file = syn::parse_file("extern crate network as hidden; use hidden::client as client;")
        .expect("parse alias chain");
    let imports = ImportMap::from_file(&file);
    let path = syn::parse_str::<syn::Path>("client::connect").expect("parse path");

    let (resolved, quality) = imports.resolve(&path);

    assert_eq!(resolved, "network::client::connect");
    assert_eq!(quality, AnalysisQuality::Exact);
}

#[test]
fn alias_cycles_are_reported_as_unresolved() {
    let file = syn::parse_file("use b as a; use a as b;").expect("parse alias cycle");
    let imports = ImportMap::from_file(&file);
    let path = syn::parse_str::<syn::Path>("a::call").expect("parse path");

    let (_, quality) = imports.resolve(&path);

    assert_eq!(quality, AnalysisQuality::Unresolved);
}

#[test]
fn self_imports_bind_the_parent_path_instead_of_a_synthetic_self_segment() {
    let file = syn::parse_file("use std::net::{self, TcpStream}; use std::fs::{self as files};")
        .expect("parse self imports");
    let imports = ImportMap::from_file(&file);

    let net = syn::parse_str::<syn::Path>("net::TcpListener").expect("parse net path");
    let files = syn::parse_str::<syn::Path>("files::read").expect("parse fs path");

    assert_eq!(
        imports.resolve(&net),
        ("std::net::TcpListener".into(), AnalysisQuality::Exact)
    );
    assert_eq!(
        imports.resolve(&files),
        ("std::fs::read".into(), AnalysisQuality::Exact)
    );
}

#[test]
fn same_root_imports_do_not_rewrite_already_qualified_paths() {
    let file = syn::parse_file("use quote::quote;").expect("parse same-root import");
    let imports = ImportMap::from_file(&file);
    let bare = syn::parse_str("quote").expect("parse bare path");
    let qualified = syn::parse_str("quote::quote::quote").expect("parse qualified path");

    assert_eq!(imports.resolve(&bare).0, "quote::quote");
    assert_eq!(imports.resolve(&qualified).0, "quote::quote::quote");
}

#[test]
fn exact_alias_expansion_has_fixed_depth_and_byte_limits() {
    let deep = (0..140).fold(String::new(), |mut source, index| {
        write!(source, "use a{} as a{index};", index + 1).expect("append alias");
        source
    });
    let imports = ImportMap::from_file(&syn::parse_file(&deep).expect("parse deep aliases"));
    let path = syn::parse_str::<syn::Path>("a0::call").expect("parse deep alias call");
    assert_eq!(imports.resolve(&path).1, AnalysisQuality::Unresolved);

    let long = std::iter::repeat_n("segment", 160)
        .collect::<Vec<_>>()
        .join("::");
    let source = format!("use {long} as bounded;");
    let imports = ImportMap::from_file(&syn::parse_file(&source).expect("parse long alias"));
    let path = syn::parse_str::<syn::Path>("bounded::call").expect("parse bounded alias call");
    assert_eq!(imports.resolve(&path).1, AnalysisQuality::Unresolved);
}

#[test]
fn conditional_top_level_imports_are_never_exact_authority() {
    let file = syn::parse_file("#[cfg(any())] use tokio as rt;").expect("parse conditional import");
    let imports = ImportMap::from_file(&file);
    let path = syn::parse_str::<syn::Path>("rt::select").expect("parse aliased macro path");

    assert_eq!(
        imports.resolve(&path),
        ("tokio::select".into(), AnalysisQuality::Unresolved)
    );
}
