//! Agent explanations expose the effective rails for one concrete path.

use std::path::Path;

use zrail_rust::explain_path;

#[test]
fn explanation_contains_actionable_source_policy() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/good");
    let explanation = explain_path(
        &root,
        Path::new("zrail.toml"),
        Path::new("crates/fixture/src/worker.rs"),
    )
    .expect("explain fixture path");

    assert_eq!(explanation.schema, 4);
    assert_eq!(explanation.reachability, "production");
    assert_eq!(explanation.unsafe_code, "deny");
    assert_eq!(explanation.lint_suppressions, "deny");
    assert_eq!(explanation.denied_methods, ["unwrap", "expect"]);
    assert_eq!(explanation.macro_expansion, "allow");
    assert_eq!(
        explanation.expected_sibling_test.as_deref(),
        Some("crates/fixture/src/worker_test.rs")
    );
}

#[test]
fn nested_module_and_include_edges_inherit_test_only_reachability() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/nested_test_context");
    for path in [
        "src/tests/support.rs",
        "src/included.rs",
        "src/tests/outer/inner/support.rs",
        "src/function.rs",
        "src/const.rs",
        "src/impl.rs",
        "src/method.rs",
        "src/function_support.rs",
        "src/file_context.rs",
        "src/file_inner.rs",
    ] {
        let explanation = explain_path(&root, Path::new("zrail.toml"), Path::new(path))
            .expect("explain nested test support");
        assert_eq!(explanation.reachability, "test-only", "{path}");
    }
}
