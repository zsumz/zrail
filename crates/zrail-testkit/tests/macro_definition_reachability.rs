//! Package macro shadowing preserves production and test-only definition domains.

use std::{fmt::Write as _, fs, path::PathBuf};

use zrail_core::Report;
use zrail_rust::check_repository;

#[test]
fn test_only_local_definition_does_not_shadow_a_production_compiler_macro() {
    let root = repository(
        "guarded-compiler",
        MANIFEST,
        &[("src/lib.rs", GUARDED_COMPILER)],
        &compiler_allowances(&["assert"]),
    );

    assert_no_macro_findings(&check(&root));
    reset(&root);
}

#[test]
fn test_only_source_target_does_not_shadow_a_production_compiler_macro() {
    let root = repository(
        "test-target-compiler",
        MANIFEST,
        &[
            ("src/lib.rs", TEST_TARGET_ROOT),
            ("src/worker_test.rs", TEST_TARGET_DEFINITION),
        ],
        &compiler_allowances(&["assert"]),
    );

    assert_no_macro_findings(&check(&root));
    reset(&root);
}

#[test]
fn test_only_local_definition_does_not_shadow_an_exact_dependency_macro() {
    let root = repository(
        "guarded-dependency",
        DEPENDENCY_MANIFEST,
        &[("src/lib.rs", GUARDED_DEPENDENCY)],
        DEPENDENCY_ALLOWANCE,
    );

    assert_no_macro_findings(&check(&root));
    reset(&root);
}

#[test]
fn production_local_definition_still_shadows_a_compiler_macro() {
    let root = repository(
        "ordinary-shadow",
        MANIFEST,
        &[("src/lib.rs", ORDINARY_SHADOW)],
        &compiler_allowances(&["assert"]),
    );

    assert_macro_findings(&check(&root), &["assert"]);
    reset(&root);
}

#[test]
fn test_invocation_sees_ordinary_and_test_only_definitions() {
    let root = repository(
        "test-namespace",
        MANIFEST,
        &[("src/lib.rs", TEST_NAMESPACE)],
        &compiler_allowances(&["assert", "panic"]),
    );

    assert_macro_findings(&check(&root), &["assert", "panic"]);
    reset(&root);
}

fn repository(name: &str, manifest: &str, files: &[(&str, &str)], allowances: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "zrail-macro-definition-{name}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create source root");
    write(&root, "Cargo.toml", manifest);
    write(&root, "zrail.toml", &format!("{CONTRACT}{allowances}"));
    for (path, contents) in files {
        write(&root, path, contents);
    }
    root
}

fn check(root: &std::path::Path) -> Report {
    check_repository(root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check macro definition fixture")
        .report
}

fn assert_no_macro_findings(report: &Report) {
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.id.starts_with("RUST-MACRO-")),
        "{}",
        report.human()
    );
}

fn assert_macro_findings(report: &Report, names: &[&str]) {
    for name in names {
        assert!(
            report.findings.iter().any(|finding| {
                finding.id.starts_with("RUST-MACRO-") && finding.message.contains(name)
            }),
            "missing {name:?}: {}",
            report.human()
        );
    }
}

fn compiler_allowances(names: &[&str]) -> String {
    let mut allowances = String::new();
    for name in names {
        writeln!(
            allowances,
            "\n[[source.rust.macros.allow]]\nname = \"{name}\"\nreason = \"Reviewed compiler expansion.\""
        )
        .expect("write compiler allowance");
    }
    allowances
}

fn write(root: &std::path::Path, path: &str, contents: &str) {
    fs::write(root.join(path), contents).expect("write fixture");
}

fn reset(root: &PathBuf) {
    if root.exists() {
        fs::remove_dir_all(root).expect("reset fixture");
    }
}

const MANIFEST: &str = "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n";
const DEPENDENCY_MANIFEST: &str = concat!(
    "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
    "[dependencies]\nreviewed_json = { package = \"serde_json\", version = \"1\" }\n",
);

const GUARDED_COMPILER: &str = r"//! Library.
#[cfg(test)]
macro_rules! assert { ($($tokens:tt)*) => {}; }
pub fn run() { assert!(true); }
";

const TEST_TARGET_ROOT: &str = concat!(
    "//! Library.\n",
    "#[cfg(test)]\n",
    "mod worker",
    "_test;\n",
    "pub fn run() { assert!(true); }\n",
);

const TEST_TARGET_DEFINITION: &str = r"//! Test support.
macro_rules! assert { ($($tokens:tt)*) => {}; }
";

const GUARDED_DEPENDENCY: &str = r#"//! Library.
use reviewed_json::json;
#[cfg(test)]
macro_rules! json { ($($tokens:tt)*) => {}; }
pub fn run() { let _ = json!({"ok": true}); }
"#;

const ORDINARY_SHADOW: &str = r"//! Library.
macro_rules! assert { ($($tokens:tt)*) => {}; }
pub fn run() { assert!(true); }
";

const TEST_NAMESPACE: &str = r#"//! Library.
macro_rules! assert { ($($tokens:tt)*) => {}; }
#[cfg(test)]
macro_rules! panic { ($($tokens:tt)*) => {}; }
#[cfg(test)]
fn proof() { assert!(true); panic!("boom"); }
"#;

const DEPENDENCY_ALLOWANCE: &str = r#"
[[source.rust.macros.allow]]
name = "serde_json::json"
inputs = "opaque"
reason = "Reviewed dependency expansion."
[source.rust.macros.allow.source]
kind = "registry"
requirement = "1"
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
module_docs = "required"
facades = "allow"
tests = "allow"
[source.rust.macros]
mode = "deny-unreviewed"
[source.rust.hygiene]
unsafe = "deny"
lint_suppressions = "allow"
"#;
