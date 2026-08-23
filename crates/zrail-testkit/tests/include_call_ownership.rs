//! Include-spliced imports cannot hide direct-call ownership evidence.

use std::{fs, path::Path};

use zrail_core::Report;
use zrail_rust::{build_lock, check_repository};

#[path = "include_call_ownership/module_reexports.rs"]
mod module_reexports;
#[path = "include_call_ownership/namespace_deep_audit.rs"]
mod namespace_deep_audit;
#[path = "include_call_ownership/ordinary_qualified.rs"]
mod ordinary_qualified;
#[path = "include_call_ownership/qualified.rs"]
mod qualified;

#[test]
fn caller_alias_used_by_included_call_cannot_bypass_call_ownership() {
    let root = fixture("caller-alias", "");
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\nuse std::process::Command as Spawn;\ninclude!(\"body.rs\");\n",
    );
    write(
        &root,
        "src/body.rs",
        "pub fn hidden() { let _ = Spawn::new(\"sh\"); }\n",
    );
    write_executor(&root);
    write_lock(&root);

    assert_owned_call(&check(&root), "process-spawn", "src/body.rs");
    reset(&root);
}

#[test]
fn included_alias_used_by_caller_cannot_bypass_call_ownership() {
    let root = fixture("included-alias", "");
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\ninclude!(\"imports.rs\");\npub fn hidden() { let _ = Spawn::new(\"sh\"); }\n",
    );
    write(
        &root,
        "src/imports.rs",
        "use std::process::Command as Spawn;\n",
    );
    write_executor(&root);
    write_lock(&root);

    assert_owned_call(&check(&root), "process-spawn", "src/lib.rs");
    reset(&root);
}

#[test]
fn test_only_include_alias_changes_only_the_test_compilation() {
    let root = fixture("test-alias", PRODUCTION_OWNER);
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\n#[cfg(not(test))]\nuse crate::Benign as Spawn;\n#[cfg(test)]\ninclude!(\"test_imports.rs\");\npub fn hidden() { let _ = Spawn::new(\"sh\"); }\npub struct Benign;\n",
    );
    write(
        &root,
        "src/test_imports.rs",
        "use std::process::Command as Spawn;\n",
    );
    write_executor(&root);
    write_lock(&root);

    let report = check(&root);
    assert_owned_call(&report, "process-spawn", "src/lib.rs");
    assert_no_owned_call(&report, "production-process", "src/lib.rs");
    reset(&root);
}

#[test]
fn repeated_nested_includes_keep_distinct_alias_environments() {
    let root = fixture("nested-occurrences", "");
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\nmod process { use std::process::Command as Spawn; include!(\"outer.rs\"); }\nmod file { use std::fs::File as Spawn; include!(\"outer.rs\"); }\n",
    );
    write(&root, "src/outer.rs", "include!(\"body.rs\");\n");
    write(
        &root,
        "src/body.rs",
        "pub fn hidden() { let _ = Spawn::new(\"sh\"); }\n",
    );
    write_executor(&root);
    write_lock(&root);

    assert_owned_call(&check(&root), "process-spawn", "src/body.rs");
    reset(&root);
}

#[test]
fn expression_include_block_alias_does_not_escape_its_block() {
    let root = fixture("expression-scope", "");
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\nuse std::fs::File as Spawn;\npub fn run() { let _ = include!(\"expr.rs\"); let _ = Spawn::open(\"file\"); }\n",
    );
    write(
        &root,
        "src/expr.rs",
        "{ use std::process::Command as Spawn; let _ = Spawn::new(\"true\"); 0 }\n",
    );
    write_executor(&root);
    write_lock(&root);

    let report = check(&root);
    assert_owned_call(&report, "process-spawn", "src/expr.rs");
    assert_no_owned_call(&report, "process-spawn", "src/lib.rs");
    reset(&root);
}

#[test]
fn included_globs_and_type_aliases_cannot_hide_owned_calls() {
    for (name, declaration, call) in [
        (
            "included-glob",
            "use std::process::*;\n",
            "Command::new(\"sh\")",
        ),
        (
            "included-type-alias",
            "type Spawn = std::process::Command;\n",
            "Spawn::new(\"sh\")",
        ),
    ] {
        let root = fixture(name, "");
        write(
            &root,
            "src/lib.rs",
            &format!(
                "//! Library.\nmod executor;\ninclude!(\"imports.rs\");\npub fn hidden() {{ let _ = {call}; }}\n"
            ),
        );
        write(&root, "src/imports.rs", declaration);
        write_executor(&root);
        write_lock(&root);

        assert_owned_call(&check(&root), "process-spawn", "src/lib.rs");
        reset(&root);
    }
}

#[test]
fn compatible_include_instances_preserve_exact_allowed_call_ownership() {
    let root = fixture("allowed-alias", "");
    write(&root, "src/lib.rs", "//! Library.\nmod executor;\n");
    write(
        &root,
        "src/executor.rs",
        "//! Authorized process owner.\ninclude!(\"imports.rs\");\npub fn allowed() { let _ = Spawn::new(\"true\"); }\n",
    );
    write(
        &root,
        "src/imports.rs",
        "use std::process::Command as Spawn;\n",
    );
    write_lock(&root);

    let report = check(&root);
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| { finding.id.starts_with("OWN-") && finding.rule == "process-spawn" }),
        "{}",
        report.human()
    );
    reset(&root);
}

fn fixture(name: &str, extra_contract: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!(
        "zrail-include-call-{name}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create fixture");
    write(&root, "Cargo.toml", MANIFEST);
    write(&root, "zrail.toml", &format!("{CONTRACT}{extra_contract}"));
    root
}

fn check(root: &Path) -> Report {
    check_repository(root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check include call fixture")
        .report
}

fn write_lock(root: &Path) {
    build_lock(root, "zrail.toml".as_ref())
        .expect("build include call lock")
        .write(&root.join("zrail.lock"))
        .expect("write include call lock");
}

fn assert_owned_call(report: &Report, rule: &str, path: &str) {
    assert!(
        report.findings.iter().any(|finding| {
            finding.id == "OWN-003" && finding.rule == rule && finding.path.as_deref() == Some(path)
        }),
        "{}",
        report.human()
    );
}

fn assert_no_owned_call(report: &Report, rule: &str, path: &str) {
    assert!(
        !report.findings.iter().any(|finding| {
            finding.id == "OWN-003" && finding.rule == rule && finding.path.as_deref() == Some(path)
        }),
        "{}",
        report.human()
    );
}

fn assert_no_owner_findings(report: &Report, rule: &str) {
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.id.starts_with("OWN-") && finding.rule == rule),
        "{}",
        report.human()
    );
}

fn write_executor(root: &Path) {
    write(
        root,
        "src/executor.rs",
        "//! Authorized process owner.\npub fn allowed() { let _ = std::process::Command::new(\"true\"); }\n",
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
mode = "allow"
[source.rust.hygiene]
unsafe = "deny"
lint_suppressions = "allow"

[[owner]]
name = "process-spawn"
kind = "call"
within = ["src/**"]
match = "std::process::Command::new"
allow = ["src/executor.rs"]
reason = "Only the executor may construct child processes."
"#;

const PRODUCTION_OWNER: &str = r#"
[[owner]]
name = "production-process"
kind = "call"
reachability = "production"
within = ["src/**"]
match = "std::process::Command::new"
allow = ["src/executor.rs"]
reason = "Only production process calls use this owner."
"#;
