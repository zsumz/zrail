//! Returning frozen expressions from a minimal harness must preserve their complete Rust AST.

#[test]
fn extracted_predicates_equal_the_frozen_initializer_expressions() {
    let root = super::super::model::project();
    let source = std::fs::read_to_string(
        root.join("crates/zrail-testkit/tests/fixtures/rc9/declarations/release_graph.rs"),
    )
    .expect("frozen detector");
    assert_eq!(
        zrail_core::sha256_hex(source.as_bytes()),
        "e30e48c6f261690844c9db7ef14d02316740c4b4d8eec322241268d454c947ff"
    );
    let source = syn::parse_file(&source).expect("frozen detector AST");
    let harness =
        std::fs::read_to_string(root.join("crates/zrail-rust/tests/rc9_declarations/legacy.rs"))
            .expect("trusted harness");
    let harness = syn::parse_file(&harness).expect("harness AST");
    for (original, binding, extracted) in [
        (
            "legacy_transport_module_trees_are_absent",
            "declared",
            "modules",
        ),
        (
            "reactor_backend_and_construction_have_no_legacy_path",
            "has_legacy_variant",
            "has_variant",
        ),
    ] {
        let expected = function(&source, original)
            .block
            .stmts
            .iter()
            .find_map(|stmt| {
                let syn::Stmt::Local(local) = stmt else {
                    return None;
                };
                let syn::Pat::Ident(ident) = &local.pat else {
                    return None;
                };
                (ident.ident == binding)
                    .then(|| local.init.as_ref().expect("initializer").expr.as_ref())
            })
            .expect("original observed expression");
        let Some(syn::Stmt::Expr(actual, None)) = function(&harness, extracted).block.stmts.last()
        else {
            panic!("extracted expression must be the unchanged return value")
        };
        assert_eq!(
            actual, expected,
            "{binding}: whole selection expression, including collection and quantity semantics"
        );
    }
}

fn function<'a>(file: &'a syn::File, name: &str) -> &'a syn::ItemFn {
    file.items
        .iter()
        .find_map(|item| match item {
            syn::Item::Fn(function) if function.sig.ident == name => Some(function),
            _ => None,
        })
        .expect("exact source function")
}
