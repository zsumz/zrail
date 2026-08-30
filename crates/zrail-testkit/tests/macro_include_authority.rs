//! Included tokens must retain exact textual macro authority across physical files.

use std::{fs, path::Path};

use zrail_core::{Report, ReportStatus};
use zrail_rust::{build_lock, check_repository};

#[test]
fn caller_definition_shadows_external_macro_inside_include() {
    let root = fixture("caller-shadow", DEPENDENCY_MANIFEST, EXTERNAL_ONLY);
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmacro_rules! json { ($($tokens:tt)*) => { 1 }; }\ninclude!(\"body.rs\");\n",
    );
    write(&root, "src/body.rs", EXTERNAL_BODY);
    write_lock(&root);

    assert_macro_failure(&check(&root), "json");
    reset(&root);
}

#[test]
fn test_only_caller_definition_is_a_content_bound_second_origin() {
    let root = fixture(
        "test-shadow",
        DEPENDENCY_MANIFEST,
        &format!("{EXTERNAL_ONLY}{LOCAL_JSON}"),
    );
    write(&root, "Cargo.lock", CARGO_LOCK);
    let source = "//! Library.\n#[cfg(test)]\nmacro_rules! json { ($($tokens:tt)*) => { 1 }; }\ninclude!(\"body.rs\");\n";
    write(&root, "src/lib.rs", source);
    write(&root, "src/body.rs", EXTERNAL_BODY);
    write_lock(&root);
    let report = check(&root);
    assert_eq!(report.status, ReportStatus::Pass, "{}", report.human());

    write(&root, "src/lib.rs", &source.replace("=> { 1 }", "=> { 2 }"));
    let report = check(&root);
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.id == "LOCK-023" && finding.message.contains("fixture")),
        "{}",
        report.human()
    );
    reset(&root);
}

#[test]
fn included_definition_shadows_compiler_macro_after_include() {
    let root = fixture("included-shadow", MANIFEST, COMPILER_ASSERT);
    write(
        &root,
        "src/lib.rs",
        "//! Library.\ninclude!(\"defs.rs\");\npub fn run() { assert!(true); }\n",
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
fn caller_definition_after_include_is_not_visible_inside_it() {
    let root = fixture("definition-after", MANIFEST, CONSERVATIVE_ASSERT);
    write(
        &root,
        "src/lib.rs",
        "//! Library.\ninclude!(\"body.rs\");\nmacro_rules! assert { ($($tokens:tt)*) => { 1 }; }\n",
    );
    write(&root, "src/body.rs", "pub fn run() { assert!(true); }\n");
    write_lock(&root);

    assert_no_macro_failure(&check(&root));
    reset(&root);
}

fn fixture(name: &str, manifest: &str, allowances: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!(
        "zrail-macro-include-{name}-{}-{:?}",
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
        .expect("check include fixture")
        .report
}

fn write_lock(root: &Path) {
    build_lock(root, "zrail.toml".as_ref())
        .expect("build include authority lock")
        .write(&root.join("zrail.lock"))
        .expect("write include authority lock");
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
const CARGO_LOCK: &str = r#"version = 4
[[package]]
name = "fixture"
version = "0.0.0"
dependencies = ["serde_json"]
[[package]]
name = "serde_json"
version = "1.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
"#;
const EXTERNAL_BODY: &str = r#"use reviewed_json::json;
pub fn run() { let _ = json!({"ok": true}); }
"#;

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

const EXTERNAL_ONLY: &str = r#"
[[source.rust.macros.allow]]
name = "serde_json::json"
inputs = "opaque"
reason = "Reviewed registry macro expansion."
[source.rust.macros.allow.source]
kind = "registry"
requirement = "1"
"#;

const LOCAL_JSON: &str = r#"
[[source.rust.macros.allow]]
name = "json"
definition = "src/lib.rs"
inputs = "opaque"
reason = "Reviewed test-domain local expansion."
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
