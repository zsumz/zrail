//! Update authority comes from reviewed Git state, never mutable worktree bytes.

use std::{fs, path::PathBuf};

use zrail_rust::build_lock;

use crate::app::{
    args::{CommonOptions, UpdateOptions},
    commands::git_base::{commit_all, git_available},
    output::OutputFormat,
};

use super::super::update::update;

#[test]
fn missing_worktree_lock_cannot_authorize_a_weaker_contract() {
    if !git_available() {
        return;
    }
    let root = adopted_fixture("missing");
    fs::remove_file(root.join("zrail.lock")).expect("remove lock");
    weaken_contract(&root);

    let refused = update(&options(&root)).expect("evaluate missing lock");

    assert_eq!(refused.exit_code, 1);
    assert!(
        refused
            .text
            .contains("UNKNOWN lock.authority before:missing")
    );
    assert!(!root.join("zrail.lock").exists());
    reset(&root);
}

#[test]
fn forged_worktree_digest_cannot_hide_a_weaker_contract() {
    if !git_available() {
        return;
    }
    let root = adopted_fixture("forged");
    weaken_contract(&root);
    build_lock(&root, std::path::Path::new("zrail.toml"))
        .expect("forge candidate lock")
        .write(&root.join("zrail.lock"))
        .expect("write forged lock");

    let refused = update(&options(&root)).expect("evaluate forged digest");

    assert_eq!(refused.exit_code, 1);
    assert!(refused.text.contains("GRANT rust.unsafe"));
    reset(&root);
}

#[test]
fn grant_acceptance_cannot_replace_missing_immutable_authority() {
    let root = fixture_root("bootstrap");
    reset(&root);
    write_fixture(&root);

    let mut options = options(&root);
    let refused = update(&options).expect("refuse implicit bootstrap");
    assert_eq!(refused.exit_code, 1);
    assert!(
        refused
            .text
            .contains("UNKNOWN lock.authority before:missing")
    );

    options.accept_grants = true;
    let refused = update(&options).expect("refuse missing immutable base");
    assert_eq!(refused.exit_code, 1);
    assert!(refused.text.contains("immutable architecture authority"));
    assert!(!root.join("zrail.lock").exists());
    reset(&root);
}

fn adopted_fixture(name: &str) -> PathBuf {
    let root = fixture_root(name);
    reset(&root);
    write_fixture(&root);
    build_lock(&root, std::path::Path::new("zrail.toml"))
        .expect("build adopted lock")
        .write(&root.join("zrail.lock"))
        .expect("write adopted lock");
    commit_all(&root);
    root
}

fn weaken_contract(root: &std::path::Path) {
    let contract = fs::read_to_string(root.join("zrail.toml")).expect("read contract");
    fs::write(
        root.join("zrail.toml"),
        contract.replace("unsafe = \"deny\"", "unsafe = \"allow\""),
    )
    .expect("weaken contract");
}

fn options(root: &std::path::Path) -> UpdateOptions {
    UpdateOptions {
        common: CommonOptions {
            root: root.to_path_buf(),
            config: "zrail.toml".into(),
            lock: "zrail.lock".into(),
            format: OutputFormat::Human,
            ..CommonOptions::default()
        },
        base: "HEAD".into(),
        accept_grants: false,
        accept_migration: None,
        migration_report: None,
    }
}

fn write_fixture(root: &std::path::Path) {
    fs::create_dir_all(root.join("src")).expect("create fixture");
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
    )
    .expect("write package");
    fs::write(root.join("src/lib.rs"), "//! fixture\n").expect("write source");
    fs::write(root.join("zrail.toml"), CONTRACT).expect("write contract");
}

fn fixture_root(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "zrail-update-authority-{name}-{}",
        std::process::id()
    ))
}

fn reset(root: &std::path::Path) {
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
mode = "locked"
unassigned_packages = "allow"
cycles = "deny"

[source.rust]
module_docs = "required"
facades = "declarative"
tests = "allow"

[source.rust.hygiene]
unsafe = "deny"
lint_suppressions = "allow"
deny_methods = []
deny_macros = []
"#;
