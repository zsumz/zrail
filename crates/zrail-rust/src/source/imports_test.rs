//! Alias, alias-chain, and grouped-import resolution examples.

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
