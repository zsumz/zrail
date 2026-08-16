//! Allowed expansion never silently authorizes ordinary or opaque invocation input.

use std::{fs, path::PathBuf};

use zrail_core::Report;
use zrail_rust::{build_lock, check_repository};

#[test]
fn standard_macro_inputs_retain_unsafe_effect_and_nested_macro_facts() {
    let root = repository("standard-inputs", "");
    write(
        &root,
        "src/lib.rs",
        r#"//! Standard inputs.
macro_rules! hidden { () => { 1 }; }
pub fn run() {
    let _ = format!("{}", unsafe { core::ptr::read_volatile(&0) });
    let _ = format!("{}", r#std::r#process::Command::new("sh").status().r#unwrap());
    let _ = vec![std::process::Command::new("sh")];
    let _ = vec![unsafe { core::ptr::read_volatile(&0) }; 2];
    let _ = matches!(unsafe { core::ptr::read_volatile(&0) }, 0 if std::process::id() > 0);
    assert!(true, "{}", hidden!());
    let _ = write!(&mut String::new(), "{}", unsafe { core::ptr::read_volatile(&0) });
}
"#,
    );
    write(
        &root,
        "zrail.toml",
        &format!(
            "{BASE}\n{}",
            allowances(&["format", "vec", "matches", "assert", "write"])
        ),
    );

    let report = check(&root);
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.id == "RUST-HYG-004")
    );
    assert!(
        report.findings.iter().any(|finding| {
            finding.id == "EFFECT-001" && finding.message.contains("std::process")
        })
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.id == "RUST-HYG-001")
    );
    assert!(
        report.findings.iter().any(|finding| {
            finding.id == "RUST-MACRO-001" && finding.message.contains("hidden")
        })
    );
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.id == "RUST-MACRO-003")
    );
    reset(&root);
}

#[test]
fn opaque_dsl_input_requires_separate_nonstale_authority() {
    let root = repository(
        "opaque",
        "dsl = { package = \"dsl-package\", version = \"1\" }",
    );
    write(
        &root,
        "src/lib.rs",
        "//! DSL.\npub fn run() { dsl::query!(select from events); }\n",
    );
    let allowance = "\n[[source.rust.macros.allow]]\nname = \"dsl_package::query\"\nreason = \"Reviewed expansion.\"\n[source.rust.macros.allow.source]\nkind = \"registry\"\nrequirement = \"1\"\n";
    write(&root, "zrail.toml", &format!("{BASE}{allowance}"));
    assert!(
        check(&root)
            .findings
            .iter()
            .any(|finding| finding.id == "RUST-MACRO-003")
    );

    let opaque = allowance.replace("reason =", "inputs = \"opaque\"\nreason =");
    write(&root, "zrail.toml", &format!("{BASE}{opaque}"));
    let reviewed = check(&root);
    assert!(
        !reviewed
            .findings
            .iter()
            .any(|finding| finding.id == "RUST-MACRO-003")
    );

    write(
        &root,
        "src/lib.rs",
        "//! Empty DSL.\npub fn run() { dsl::query!(); }\n",
    );
    assert!(
        check(&root)
            .findings
            .iter()
            .any(|finding| finding.id == "RUST-MACRO-004")
    );
    reset(&root);
}

#[test]
fn approved_local_macro_body_is_content_bound_in_the_lock() {
    let root = repository("local-body", "");
    let safe = "//! Local macro.\nmod local { macro_rules! reviewed { () => { 42 }; } pub(crate) use reviewed; }\npub fn run() { let _ = local::reviewed!(); }\n";
    write(&root, "src/lib.rs", safe);
    write(
        &root,
        "zrail.toml",
        &format!(
            "{BASE}\n[[source.rust.macros.allow]]\nname = \"local::reviewed\"\ndefinition = \"src/lib.rs\"\nreason = \"Reviewed local transcriber.\"\n"
        ),
    );
    build_lock(&root, "zrail.toml".as_ref())
        .expect("build local macro lock")
        .write(&root.join("zrail.lock"))
        .expect("write local macro lock");
    assert!(
        !check(&root)
            .findings
            .iter()
            .any(|finding| finding.id.starts_with("LOCK-"))
    );

    write(
        &root,
        "src/lib.rs",
        &safe.replace("42", "unsafe { core::ptr::read_volatile(&0) }"),
    );
    assert!(
        check(&root)
            .findings
            .iter()
            .any(|finding| finding.id == "LOCK-019")
    );
    reset(&root);
}

fn allowances(names: &[&str]) -> String {
    names.iter().fold(String::new(), |mut contract, name| {
        contract.push_str("[[source.rust.macros.allow]]\nname = \"");
        contract.push_str(name);
        contract.push_str("\"\nreason = \"Reviewed standard expansion.\"\n\n");
        contract
    })
}

fn repository(name: &str, dependency: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "zrail-macro-input-{name}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create fixture");
    write(
        &root,
        "Cargo.toml",
        &format!(
            "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\n[dependencies]\n{dependency}\n"
        ),
    );
    root
}

fn check(root: &std::path::Path) -> Report {
    check_repository(root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check macro input fixture")
        .report
}

fn write(root: &std::path::Path, path: &str, contents: &str) {
    fs::write(root.join(path), contents).expect("write fixture");
}

fn reset(root: &PathBuf) {
    if root.exists() {
        fs::remove_dir_all(root).expect("reset fixture");
    }
}

const BASE: &str = r#"schema = 1
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
deny_methods = ["unwrap"]
[profiles.restricted.effects]
deny = ["process"]
[[layer]]
name = "app"
packages = ["fixture"]
profiles = ["restricted"]
reason = "Fixture policy."
[layer.dependencies]
external = "allow"
"#;
