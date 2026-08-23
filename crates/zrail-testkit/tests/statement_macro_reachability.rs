//! Statement macro attributes constrain effects and ownership to their cfg domain.

use std::{fs, path::PathBuf};

use zrail_rust::check_repository;

#[test]
fn test_only_statement_macro_input_is_not_production_authority() {
    let root = repository();

    let report = check_repository(&root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check statement macro fixture")
        .report;

    assert!(
        !report.findings.iter().any(|finding| {
            finding.id == "EFFECT-001"
                && finding.path.as_deref() == Some("src/lib.rs")
                && finding.message.contains("process")
        }),
        "{}",
        report.human()
    );
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| { finding.id == "OWN-003" && finding.rule == "runtime-process" }),
        "{}",
        report.human()
    );
    assert!(
        report.findings.iter().any(|finding| {
            finding.id == "OWN-004"
                && finding.rule == "runtime-process"
                && finding.message.contains("no production-reachable use")
        }),
        "{}",
        report.human()
    );
    reset(&root);
}

fn repository() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "zrail-statement-macro-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create fixture directory");
    write(&root, "Cargo.toml", MANIFEST);
    write(&root, "src/lib.rs", SOURCE);
    write(&root, "zrail.toml", CONTRACT);
    root
}

fn write(root: &std::path::Path, path: &str, contents: &str) {
    fs::write(root.join(path), contents).expect("write fixture");
}

fn reset(root: &PathBuf) {
    if root.exists() {
        fs::remove_dir_all(root).expect("reset fixture");
    }
}

const MANIFEST: &str = r#"[package]
name = "fixture"
version = "0.0.0"
edition = "2024"
"#;

const SOURCE: &str = r#"//! Kernel.

pub fn run() {
    #[cfg(test)]
    assert!(std::process::Command::new("true").status().is_ok());
}
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
mode = "allow"
[source.rust.hygiene]
unsafe = "deny"
lint_suppressions = "allow"
[profiles.kernel]
reachability = "production"
[profiles.kernel.effects]
deny = ["process"]
[[layer]]
name = "kernel"
packages = ["fixture"]
profiles = ["kernel"]
reason = "Fixture policy."
[[owner]]
name = "runtime-process"
kind = "call"
reachability = "production"
within = ["src/**"]
match = "std::process::Command::new"
allow = ["src/lib.rs"]
reason = "Runtime process calls require an explicit owner."
"#;
