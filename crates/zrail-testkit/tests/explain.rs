//! Agent explanations expose the effective rails for one concrete path.

use std::{fs, path::Path};

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

    assert_eq!(explanation.schema, 2);
    assert_eq!(explanation.reachability, "production");
    assert_eq!(explanation.unsafe_code, "deny");
    assert_eq!(explanation.lint_suppressions, "deny");
    assert_eq!(explanation.denied_methods, ["unwrap", "expect"]);
    assert!(explanation.denied_syntax.is_empty());
    assert_eq!(explanation.glob_imports, "allow");
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

#[test]
fn explanation_separates_opaque_input_from_content_bound_expansion() {
    let root = std::env::temp_dir().join(format!(
        "zrail-explain-macro-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("src")).expect("create explanation fixture");
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
    )
    .expect("write manifest");
    fs::write(
        root.join("src/lib.rs"),
        "//! Macro policy.\nmod local { macro_rules! query { ($($input:tt)*) => { 1 }; } pub(crate) use query; }\npub fn run() { let _ = local::query!(select from events); }\n",
    )
    .expect("write source");
    fs::write(root.join("zrail.toml"), MACRO_CONTRACT).expect("write contract");

    let explanation = explain_path(&root, Path::new("zrail.toml"), Path::new("src/lib.rs"))
        .expect("explain macro policy");

    assert_eq!(explanation.opaque_macro_inputs, ["local::query"]);
    assert_eq!(
        explanation.content_bound_macro_implementations,
        ["local::query@fixture:."]
    );
    fs::remove_dir_all(root).expect("remove explanation fixture");
}

#[test]
fn explanation_separates_written_public_and_origin_macro_identity() {
    let root = std::env::temp_dir().join(format!(
        "zrail-explain-macro-name-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("src")).expect("create explanation fixture");
    fs::write(
        root.join("Cargo.toml"),
        concat!(
            "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
            "[dependencies]\nreviewed_quote = { package = \"quote\", version = \"1\" }\n",
        ),
    )
    .expect("write manifest");
    fs::write(
        root.join("src/lib.rs"),
        "//! Macro identity.\nuse reviewed_quote::quote as q;\npub fn run() { let _ = q!(); }\n",
    )
    .expect("write source");
    let contract = MACRO_CONTRACT
        .replace("name = \"local::query\"", "name = \"quote::quote\"")
        .replace("inputs = \"opaque\"\n", "")
        .replace(
            "definition = \"src/lib.rs\"\nreason = \"Reviewed local query boundary.\"",
            "reason = \"Reviewed quote boundary.\"\n[source.rust.macros.allow.source]\nkind = \"registry\"\nrequirement = \"1\"",
        );
    fs::write(root.join("zrail.toml"), contract).expect("write contract");

    let explanation = explain_path(&root, Path::new("zrail.toml"), Path::new("src/lib.rs"))
        .expect("explain macro identity");

    assert_eq!(explanation.macro_invocations.len(), 1);
    let invocation = &explanation.macro_invocations[0];
    assert_eq!(invocation.written, "q");
    assert_eq!(invocation.preferred.as_deref(), Some("quote::quote"));
    assert_eq!(invocation.origins, ["external:quote:registry:crates.io:1"]);
    assert!(
        explanation
            .human()
            .contains("q -> quote::quote @ external:quote")
    );
    fs::remove_dir_all(root).expect("remove explanation fixture");
}

const MACRO_CONTRACT: &str = r#"schema = 1
adapters = ["rust"]
[repository]
roots = ["."]
exclude = []
workspace_members = "exact"
nested_git = "deny"
submodules = "deny"
symlinks = "inside"
[dependencies]
mode = "observed"
unassigned_packages = "allow"
cycles = "deny"
[source.rust]
module_docs = "required"
facades = "allow"
tests = "allow"
[source.rust.macros]
mode = "deny-unreviewed"
[[source.rust.macros.allow]]
name = "local::query"
inputs = "opaque"
definition = "src/lib.rs"
reason = "Reviewed local query boundary."
[source.rust.hygiene]
unsafe = "deny"
lint_suppressions = "allow"
[[layer]]
name = "app"
packages = ["fixture"]
profiles = []
reason = "Fixture layer."
[layer.dependencies]
external = "allow"
"#;
