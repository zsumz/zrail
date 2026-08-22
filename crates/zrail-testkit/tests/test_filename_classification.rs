//! Conventional test filenames drive budgets, explanation, and baseline discovery.

use std::{fs, path::PathBuf};

use zrail_rust::{BaselineRule, check_repository, discover_baseline_rules, explain_path};

#[test]
fn tests_rs_uses_test_policy_across_consumers() {
    let root = repository();
    let report = check_repository(&root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check test filename fixture")
        .report;
    assert!(report.findings.iter().any(|finding| {
        finding.id == "RUST-SIZE-002" && finding.path.as_deref() == Some("src/tests.rs")
    }));

    let explanation = explain_path(&root, "zrail.toml".as_ref(), "src/tests.rs".as_ref())
        .expect("explain tests.rs");
    assert_eq!(explanation.file_class, "test");
    assert_eq!(explanation.reachability, "test-only");
    assert_eq!(explanation.design_target, Some(3));

    let baseline = discover_baseline_rules(&root, "zrail.toml".as_ref(), &[BaselineRule::FileSize])
        .expect("discover test filename debt");
    assert!(
        baseline.ratchets.iter().any(|ratchet| {
            ratchet.rule == "rust.file-size" && ratchet.target == "src/tests.rs"
        })
    );
    reset(&root);
}

fn repository() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "zrail-test-filename-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create source directory");
    write(&root, "Cargo.toml", MANIFEST);
    write(&root, "zrail.toml", CONTRACT);
    write(&root, "src/lib.rs", LIBRARY);
    write(&root, "src/tests.rs", TESTS);
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

const LIBRARY: &str = concat!("//! Library.\n", "#[cfg(test)]\n", "mod tests;\n",);

const TESTS: &str = r"//! Conventional tests.

#[test]
fn one() { assert!(true); }

#[test]
fn two() { assert!(true); }
";

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
[source.rust.size.facade]
target = 20
hard = 30
[source.rust.size.implementation]
target = 20
hard = 30
[source.rust.size.test]
target = 3
hard = 20
[source.rust.size.auxiliary]
target = 20
hard = 30
"#;
