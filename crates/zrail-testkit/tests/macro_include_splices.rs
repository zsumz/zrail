//! Include occurrences preserve nesting, lexical containment, and fail-closed aliases.

use std::{fs, path::Path};

use zrail_core::Report;
use zrail_rust::{build_lock, check_repository};

#[test]
fn repeated_include_occurrences_receive_distinct_macro_environments() {
    let root = fixture("occurrences", MANIFEST, COMPILER_ASSERT);
    write(
        &root,
        "src/lib.rs",
        "//! Library.\npub fn first() { let _ = include!(\"body.rs\"); }\nmacro_rules! assert { ($($tokens:tt)*) => { 1 }; }\npub fn second() { let _ = include!(\"body.rs\"); }\n",
    );
    write(&root, "src/body.rs", "{ assert!(true) }\n");
    write_lock(&root);

    assert_macro_failure(&check(&root), "assert");
    reset(&root);
}

#[test]
fn nested_includes_preserve_definition_order() {
    let root = fixture("nested", MANIFEST, COMPILER_ASSERT);
    write(
        &root,
        "src/lib.rs",
        "//! Library.\ninclude!(\"outer.rs\");\n",
    );
    write(
        &root,
        "src/outer.rs",
        "include!(\"defs.rs\");\npub fn run() { assert!(true); }\n",
    );
    write(
        &root,
        "src/defs.rs",
        "macro_rules! assert { ($($tokens:tt)*) => { 1 }; }\n",
    );
    write_lock(&root);

    assert_macro_failure(&check(&root), "assert");
    reset(&root);
}

#[test]
fn inline_module_include_does_not_leak_macro_definitions() {
    let root = fixture("inline-scope", MANIFEST, COMPILER_ASSERT);
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod inner { include!(\"defs.rs\"); pub fn local() { assert!(true); } }\npub fn outer() { assert!(true); }\n",
    );
    write(
        &root,
        "src/defs.rs",
        "macro_rules! assert { ($($tokens:tt)*) => { 1 }; }\n",
    );
    write_lock(&root);

    let report = check(&root);
    let macro_findings = report
        .findings
        .iter()
        .filter(|finding| finding.id.starts_with("RUST-MACRO-"))
        .collect::<Vec<_>>();
    assert_eq!(macro_findings.len(), 1, "{}", report.human());
    assert_eq!(macro_findings[0].span.map(|span| span.line), Some(2));
    reset(&root);
}

#[test]
fn expression_include_does_not_leak_macro_definitions() {
    let root = fixture("expression-scope", MANIFEST, COMPILER_ASSERT);
    write(
        &root,
        "src/lib.rs",
        "//! Library.\npub fn inner() { let _ = include!(\"body.rs\"); }\npub fn outer() { assert!(true); }\n",
    );
    write(
        &root,
        "src/body.rs",
        "{ macro_rules! assert { ($($tokens:tt)*) => { 1 }; } assert!(true) }\n",
    );
    write_lock(&root);

    let report = check(&root);
    let macro_findings = report
        .findings
        .iter()
        .filter(|finding| finding.id.starts_with("RUST-MACRO-"))
        .collect::<Vec<_>>();
    assert_eq!(macro_findings.len(), 1, "{}", report.human());
    assert_eq!(macro_findings[0].path.as_deref(), Some("src/body.rs"));
    reset(&root);
}

#[test]
fn aliases_crossing_include_splices_require_conservative_name_authority() {
    let caller_exact = fixture("caller-alias-exact", DEPENDENCY_MANIFEST, COMPILER_ASSERT);
    write(
        &caller_exact,
        "src/lib.rs",
        "//! Library.\nuse reviewed_json::json as assert;\ninclude!(\"body.rs\");\n",
    );
    write(
        &caller_exact,
        "src/body.rs",
        "pub fn run() { assert!(true); }\n",
    );
    write_lock(&caller_exact);
    assert_macro_failure(&check(&caller_exact), "assert");
    reset(&caller_exact);

    let caller_conservative = fixture(
        "caller-alias-conservative",
        DEPENDENCY_MANIFEST,
        CONSERVATIVE_ASSERT,
    );
    write(
        &caller_conservative,
        "src/lib.rs",
        "//! Library.\nuse reviewed_json::json as assert;\ninclude!(\"body.rs\");\n",
    );
    write(
        &caller_conservative,
        "src/body.rs",
        "pub fn run() { assert!(true); }\n",
    );
    write_lock(&caller_conservative);
    assert_no_macro_failure(&check(&caller_conservative));
    reset(&caller_conservative);

    let included_exact = fixture("included-alias-exact", DEPENDENCY_MANIFEST, COMPILER_ASSERT);
    write(
        &included_exact,
        "src/lib.rs",
        "//! Library.\ninclude!(\"imports.rs\");\npub fn run() { assert!(true); }\n",
    );
    write(
        &included_exact,
        "src/imports.rs",
        "use reviewed_json::json as assert;\n",
    );
    write_lock(&included_exact);
    assert_macro_failure(&check(&included_exact), "assert");
    reset(&included_exact);

    let included_conservative = fixture(
        "included-alias-conservative",
        DEPENDENCY_MANIFEST,
        CONSERVATIVE_ASSERT,
    );
    write(
        &included_conservative,
        "src/lib.rs",
        "//! Library.\ninclude!(\"imports.rs\");\npub fn run() { assert!(true); }\n",
    );
    write(
        &included_conservative,
        "src/imports.rs",
        "use reviewed_json::json as assert;\n",
    );
    write_lock(&included_conservative);
    assert_no_macro_failure(&check(&included_conservative));
    reset(&included_conservative);
}

fn fixture(name: &str, manifest: &str, allowances: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!(
        "zrail-macro-splice-{name}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create fixture");
    write(&root, "Cargo.toml", manifest);
    write(&root, "zrail.toml", &format!("{CONTRACT}{allowances}"));
    root
}

fn check(root: &Path) -> Report {
    check_repository(root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check include splice fixture")
        .report
}

fn write_lock(root: &Path) {
    build_lock(root, "zrail.toml".as_ref())
        .expect("build include splice lock")
        .write(&root.join("zrail.lock"))
        .expect("write include splice lock");
}

fn assert_macro_failure(report: &Report, name: &str) {
    assert!(
        report.findings.iter().any(|finding| {
            finding.id.starts_with("RUST-MACRO-") && finding.message.contains(name)
        }),
        "{}",
        report.human()
    );
}

fn assert_no_macro_failure(report: &Report) {
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.id.starts_with("RUST-MACRO-")),
        "{}",
        report.human()
    );
}

fn write(root: &Path, path: &str, contents: &str) {
    fs::write(root.join(path), contents).expect("write fixture");
}

fn reset(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).expect("reset fixture");
    }
}

const MANIFEST: &str = "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n";
const DEPENDENCY_MANIFEST: &str = concat!(
    "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
    "[dependencies]\nreviewed_json = { package = \"serde_json\", version = \"1\" }\n",
);

const CONTRACT: &str = r#"schema = 1
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
module_docs = "allow"
facades = "allow"
tests = "allow"
[source.rust.macros]
mode = "deny-unreviewed"
[source.rust.hygiene]
unsafe = "deny"
lint_suppressions = "allow"
"#;

const COMPILER_ASSERT: &str = r#"
[[source.rust.macros.allow]]
name = "assert"
reason = "Reviewed compiler expansion."
"#;

const CONSERVATIVE_ASSERT: &str = r#"
[[source.rust.macros.allow]]
name = "assert"
binding = "conservative"
reason = "Reviewed unresolved include-scope name."
"#;
