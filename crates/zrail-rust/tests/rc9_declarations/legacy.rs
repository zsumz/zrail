//! Frozen release-graph predicates retain direct-file selection and written identity semantics.

use syn::Item;

pub(super) fn modules(source: &str) -> Vec<String> {
    let source = source.to_owned();
    let syntax = syn::parse_file(&source).unwrap_or_else(|error| panic!("parse reactor: {error}"));
    let legacy = [
        "broker_set",
        "plaintext",
        "poller",
        "resource",
        "tcp",
        "timer",
        "tls",
    ];
    syntax
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Mod(module) if legacy.contains(&module.ident.to_string().as_str()) => {
                Some(module.ident.to_string())
            }
            _ => None,
        })
        .collect::<Vec<_>>()
}

pub(super) fn has_variant(backend: &str) -> bool {
    let backend = backend.to_owned();
    let backend =
        syn::parse_file(&backend).unwrap_or_else(|error| panic!("parse backend: {error}"));
    backend.items.iter().any(|item| match item {
        Item::Enum(item) if item.ident == "ReactorBackend" => item
            .variants
            .iter()
            .any(|variant| variant.ident == "Legacy"),
        _ => false,
    })
}

pub(super) fn check_modules(declared: &[String]) {
    assert!(
        declared.is_empty(),
        "legacy reactor modules remain declared: {declared:?}"
    );
}

pub(super) fn check_variant(has_legacy_variant: bool) {
    assert!(
        !has_legacy_variant,
        "ReactorBackend must have no legacy variant"
    );
}
