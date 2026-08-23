//! Attribute expansion extraction distinguishes inert metadata from generated Rust.

use super::{ExpansionKind, attribute_paths, is_builtin_derive, is_compiler_derive};

#[test]
fn derives_custom_attributes_and_nested_cfg_attributes_are_boundaries() {
    let item: syn::ItemStruct = syn::parse_quote! {
        #[derive(Debug, serde::Serialize)]
        #[cfg_attr(unix, tokio::main)]
        #[repr(C)]
        struct Message;
    };

    let names = item
        .attrs
        .iter()
        .flat_map(|attribute| attribute_paths(attribute).expect("parse attribute expansion"))
        .map(|expansion| {
            expansion
                .path
                .segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect::<Vec<_>>()
                .join("::")
        })
        .collect::<Vec<_>>();

    assert_eq!(names, ["Debug", "serde::Serialize", "tokio::main"]);
}

#[test]
fn only_unqualified_standard_derives_are_compiler_builtins() {
    let item: syn::ItemStruct = syn::parse_quote! {
        #[derive(Debug, dependency::Debug)]
        struct Message;
    };
    let expansions = attribute_paths(&item.attrs[0]).expect("parse derive expansions");

    assert_eq!(expansions[0].kind, ExpansionKind::Derive);
    assert!(is_builtin_derive(&expansions[0].path));
    assert!(!is_builtin_derive(&expansions[1].path));
    assert!(is_compiler_derive(&expansions[0].path, "std::fmt::Debug"));
    assert!(!is_compiler_derive(
        &expansions[0].path,
        "dependency::Debug"
    ));
}

#[test]
fn tool_and_builtin_attributes_are_inert() {
    let item: syn::ItemStruct = syn::parse_quote! {
        #[doc = "message"]
        #[clippy::msrv = "1.96"]
        #[non_exhaustive]
        struct Message;
    };

    assert!(
        item.attrs
            .iter()
            .flat_map(|attribute| attribute_paths(attribute).expect("parse inert attribute"))
            .next()
            .is_none()
    );
}

#[test]
fn malformed_derive_and_cfg_attr_syntax_is_unresolved() {
    for source in [
        "#![derive(name = value)]",
        "#![derive()]",
        "#![cfg_attr(test)]",
    ] {
        let file = syn::parse_file(source).expect("parse permissive attribute tokens");
        assert!(attribute_paths(&file.attrs[0]).is_err(), "{source}");
    }
}
