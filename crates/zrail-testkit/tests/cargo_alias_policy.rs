//! Cargo aliases cannot hide canonical source effects, symbols, macros, or owners.

use std::{fs, path::PathBuf};

use zrail_core::Finding;
use zrail_rust::check_repository;

#[test]
fn dependency_aliases_compose_with_rust_aliases_across_policy_facts() {
    let root = repository("policy");

    let report = check_repository(
        &root,
        std::path::Path::new("zrail.toml"),
        std::path::Path::new("zrail.lock"),
    )
    .expect("analyze aliased dependencies")
    .report;

    assert_finding(&report.findings, "CAP-001", "alias_use.rs");
    assert_finding(&report.findings, "EFFECT-001", "alias_use.rs");
    assert_finding(&report.findings, "EFFECT-001", "database.rs");
    assert_finding(&report.findings, "EFFECT-001", "extern_alias.rs");
    assert_finding(&report.findings, "EFFECT-001", "raw_alias.rs");
    assert_finding(&report.findings, "CAP-001", "macro_alias.rs");
    assert_finding(&report.findings, "RUST-HYG-002", "macro_alias.rs");
    assert_finding(&report.findings, "CAP-001", "shared.rs");
    assert_finding(&report.findings, "EFFECT-001", "shared.rs");
    assert_rule_finding(&report.findings, "runtime-capability", "alias_use.rs");
    assert_rule_finding(&report.findings, "runtime-construction", "alias_use.rs");
    assert_message(&report.findings, "alias_use.rs", "Process");
    assert_message(&report.findings, "alias_use.rs", "AsyncRuntime");
    assert_message(&report.findings, "database.rs", "Database");
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| { matches!(finding.id.as_str(), "RUST-MACRO-001" | "RUST-MACRO-002") }),
        "{:#?}",
        report.findings
    );
    assert!(report.findings.iter().any(|finding| {
        finding
            .path
            .as_deref()
            .is_some_and(|path| path.ends_with("alias_use.rs"))
            && finding.message.contains("async_runtime::process")
    }));
    reset(&root);
}

fn assert_rule_finding(findings: &[Finding], rule: &str, file: &str) {
    assert!(
        findings.iter().any(|finding| {
            finding.id == "OWN-003"
                && finding.rule == rule
                && finding
                    .path
                    .as_deref()
                    .is_some_and(|path| path.ends_with(file))
        }),
        "missing {rule} ownership finding for {file}: {findings:#?}"
    );
}

fn assert_message(findings: &[Finding], file: &str, message: &str) {
    assert!(
        findings.iter().any(|finding| {
            finding.id == "EFFECT-001"
                && finding.message.contains(message)
                && finding
                    .path
                    .as_deref()
                    .is_some_and(|path| path.ends_with(file))
        }),
        "missing {message} effect finding for {file}: {findings:#?}"
    );
}

fn assert_finding(findings: &[Finding], id: &str, file: &str) {
    assert!(
        findings.iter().any(|finding| {
            finding.id == id
                && finding
                    .path
                    .as_deref()
                    .is_some_and(|path| path.ends_with(file))
        }),
        "missing {id} for {file}: {findings:#?}"
    );
}

fn repository(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "zrail-alias-{name}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    fs::create_dir_all(root.join("crates/app/src")).expect("create package source");
    fs::write(root.join("Cargo.toml"), WORKSPACE).expect("write workspace");
    fs::write(root.join("crates/app/Cargo.toml"), MANIFEST).expect("write package manifest");
    fs::write(root.join("zrail.toml"), CONTRACT).expect("write contract");
    for (path, source) in SOURCES {
        fs::write(root.join(path), source).expect("write aliased source");
    }
    root
}

fn reset(root: &PathBuf) {
    if root.exists() {
        fs::remove_dir_all(root).expect("reset fixture");
    }
}

const WORKSPACE: &str = r#"[workspace]
members = ["crates/app"]
resolver = "3"
"#;

const MANIFEST: &str = r#"[package]
name = "app"
version = "0.0.0"
edition = "2024"

[dependencies]
async-runtime = { package = "tokio", version = "1", features = ["process"] }
db = { package = "sqlx", version = "1" }
async = { package = "smol", version = "2" }
"#;

const SOURCES: &[(&str, &str)] = &[
    (
        "crates/app/src/lib.rs",
        "//! Aliased dependency fixture.\nmod adapter;\nmod alias_use;\nmod database;\nmod extern_alias;\nmod macro_alias;\nmod raw_alias;\ninclude!(\"../../shared.rs\");\n",
    ),
    (
        "crates/app/src/adapter.rs",
        "//! Declared owner.\npub(crate) fn run() { async_runtime::process::Command::new(\"sh\"); }\n",
    ),
    (
        "crates/app/src/alias_use.rs",
        "//! Function-local Rust alias over a hyphenated Cargo alias.\npub(crate) fn run() { use async_runtime as rt; rt::process::Command::new(\"sh\"); }\n",
    ),
    (
        "crates/app/src/database.rs",
        "//! Database alias.\nuse db as storage;\npub(crate) fn query() { let _ = storage::query; }\n",
    ),
    (
        "crates/app/src/extern_alias.rs",
        "//! Extern crate alias.\nextern crate async_runtime as executor;\npub(crate) fn spawn() { executor::spawn(); }\n",
    ),
    (
        "crates/app/src/macro_alias.rs",
        "//! Function-local macro alias.\npub(crate) fn choose() { use async_runtime as rt; rt::select! {} }\n",
    ),
    (
        "crates/app/src/raw_alias.rs",
        "//! Raw identifier Cargo alias.\npub(crate) fn spawn() { r#async::spawn(); }\n",
    ),
    (
        "crates/shared.rs",
        "//! Shared source outside the package directory.\npub(crate) fn run() { async_runtime::process::Command::new(\"sh\"); }\n",
    ),
];

const CONTRACT: &str = r#"schema = 1
adapters = ["rust"]

[repository]
roots = ["crates"]
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
facades = "declarative"
tests = "allow"

[source.rust.macros]
mode = "deny-unreviewed"

[[source.rust.macros.allow]]
name = "tokio::select"
reason = "The fixture explicitly reviews the aliased runtime selection expansion."
[source.rust.macros.allow.source]
kind = "registry"
requirement = "1"

[source.rust.hygiene]
unsafe = "deny"
lint_suppressions = "allow"
deny_methods = []
deny_macros = ["tokio::select"]

[profiles.pure.effects]
deny = ["process", "async-runtime", "database"]

[[layer]]
name = "application"
packages = ["app"]
profiles = ["pure"]
reason = "The fixture package is effect-free."

[[scope]]
name = "canonical-symbols"
include = ["crates/app/src/**", "crates/shared.rs"]
reason = "Canonical dependency paths remain policy-visible."

[scope.symbols]
deny = ["tokio::process", "tokio::select"]

[[owner]]
name = "runtime-capability"
kind = "capability"
within = ["crates/app/src/**"]
match = "tokio::process"
allow = ["crates/app/src/adapter.rs"]
reason = "Only the adapter owns process runtime access."

[[owner]]
name = "runtime-construction"
kind = "call"
within = ["crates/app/src/**"]
match = "tokio::process::Command::new"
allow = ["crates/app/src/adapter.rs"]
reason = "Only the adapter constructs processes."
"#;
