//! Standard and opaque macro inputs retain nested Rust policy facts.

use std::{
    fs,
    path::{Path, PathBuf},
};

use zrail_core::{Finding, Report};
use zrail_rust::check_repository;

#[test]
fn matches_pattern_macros_are_reviewed_independently() {
    let root = repository("matches-pattern", "");
    write(
        &root,
        "src/lib.rs",
        r"//! Pattern macro.
macro_rules! hidden { () => { Some(_) }; }
pub fn run(value: Option<u8>) { let _ = matches!(value, hidden!()); }
",
    );
    write(
        &root,
        "zrail.toml",
        &format!("{CONTRACT}\n{}", allowance("matches", false, "")),
    );

    assert_finding(&check(&root).findings, "RUST-MACRO-001", "hidden");
    reset(&root);
}

#[test]
fn opaque_input_compiler_macros_retain_effects_and_path_validation() {
    let root = repository(
        "opaque-effects",
        "dsl = { package = \"dsl-package\", version = \"1\" }",
    );
    write(
        &root,
        "src/hidden.rs",
        "unsafe { core::ptr::read_volatile(&0) }",
    );
    write(
        &root,
        "src/lib.rs",
        r#"//! Opaque compiler effects.
pub fn run() {
    dsl::query!(env!("HOME"), option_env!("USER"), include!("hidden.rs"), include_str!("../../outside.txt"), include_bytes!("../../outside.bin"));
}
"#,
    );
    write(
        &root,
        "zrail.toml",
        &format!(
            "{CONTRACT}\n{}",
            allowance(
                "dsl_package::query",
                true,
                "[source.rust.macros.allow.source]\nkind = \"registry\"\nrequirement = \"1\"\n",
            )
        ),
    );

    let report = check(&root);
    for effect in ["CompileEnvironment", "CompileFilesystem"] {
        assert_finding(&report.findings, "EFFECT-001", effect);
    }
    assert_finding(&report.findings, "RUST-COMPILE-001", "include!");
    assert_finding(&report.findings, "RUST-COMPILE-001", "include_str!");
    assert_finding(&report.findings, "RUST-COMPILE-001", "include_bytes!");
    reset(&root);
}

fn repository(name: &str, dependency: &str) -> PathBuf {
    let root =
        std::env::temp_dir().join(format!("zrail-macro-nested-{name}-{}", std::process::id()));
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create fixture");
    write(
        &root,
        "Cargo.toml",
        &format!(
            "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n[dependencies]\n{dependency}\n"
        ),
    );
    root
}

fn allowance(name: &str, opaque: bool, source: &str) -> String {
    format!(
        "[[source.rust.macros.allow]]\nname = \"{name}\"\n{}reason = \"Reviewed expansion boundary.\"\n{source}",
        if opaque { "inputs = \"opaque\"\n" } else { "" },
    )
}

fn check(root: &Path) -> Report {
    check_repository(root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check nested macro fixture")
        .report
}

fn assert_finding(findings: &[Finding], id: &str, text: &str) {
    assert!(
        findings
            .iter()
            .any(|finding| finding.id == id && finding.message.contains(text)),
        "missing {id} for {text}: {findings:#?}"
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
[profiles.restricted.effects]
deny = ["compile-environment", "compile-filesystem"]
[[layer]]
name = "app"
packages = ["fixture"]
profiles = ["restricted"]
reason = "Fixture policy."
"#;
