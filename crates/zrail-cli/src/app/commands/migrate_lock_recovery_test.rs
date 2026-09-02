//! Migration base recovery is explicit, read-only, and digest-bound.

use std::fs;

use crate::app::args::MigrateLockOptions;

use super::{super::migrate_lock::migrate_lock, discover};
use crate::app::commands::git_base::{
    CONTRACT_PREFIX, CONTRACT_SUFFIX, commit_all, commit_index, fixture_root, git_available,
    head_revision, reset, run_git,
};

#[test]
fn discovery_lists_every_matching_lock_revision_without_selecting_one() {
    if !git_available() {
        return;
    }
    let root = recovery_fixture("discover-multiple");
    let first = head_revision(&root);
    let mut lock = zrail_core::LockFile::read(&root.join("zrail.lock")).expect("read lock");
    lock.producer = "alternate-test-producer".into();
    lock.write(&root.join("zrail.lock"))
        .expect("write alternate lock producer");
    run_git(&root, &["add", "zrail.lock"]);
    commit_index(&root);
    let second = head_revision(&root);
    fs::write(root.join("zrail.toml"), contract("src")).expect("change contract");
    run_git(&root, &["add", "zrail.toml"]);
    commit_index(&root);

    let options = discovery_options(&root);
    let output = discover(&options).expect("discover migration bases");

    assert_eq!(output.exit_code, 0);
    assert!(output.text.contains(&first));
    assert!(output.text.contains(&second));
    assert!(
        output
            .text
            .contains("No revision was selected automatically")
    );
    assert!(output.text.contains("Lock contract digest:"));
    assert!(output.text.contains("Current contract digest:"));
    assert!(
        output
            .text
            .contains("Local uncommitted contract edits contributed: no")
    );

    let mut migration = options;
    migration.discover_base = false;
    migration.output = Some("migration.json".into());
    let error = migrate_lock(&migration).expect_err("HEAD contract must not match the stale lock");
    assert!(error.message.contains("The current contract differs"));
    assert!(error.message.contains("Selected base contract digest:"));
    assert!(error.message.contains("--discover-base"));
    assert!(error.message.contains(&first));
    assert!(error.message.contains(&second));
    assert!(!root.join("migration.json").exists());
    reset(&root);
}

#[test]
fn discovery_identifies_uncommitted_contract_edits_as_the_cause() {
    if !git_available() {
        return;
    }
    let root = recovery_fixture("discover-worktree");
    let base = head_revision(&root);
    fs::write(root.join("zrail.toml"), contract("src")).expect("edit contract");

    let output = discover(&discovery_options(&root)).expect("discover migration base");

    assert_eq!(output.exit_code, 0);
    assert!(output.text.contains(&base));
    assert!(
        output
            .text
            .contains("Local uncommitted contract edits contributed: yes")
    );
    assert!(output.text.contains("HEAD still matches the lock"));
    reset(&root);
}

fn recovery_fixture(name: &str) -> std::path::PathBuf {
    let root = fixture_root(name);
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create source directory");
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname='fixture'\nversion='0.0.0'\nedition='2024'\n",
    )
    .expect("write manifest");
    fs::write(root.join("src/lib.rs"), "//! Fixture.\n").expect("write source");
    fs::write(root.join("zrail.toml"), contract(".")).expect("write contract");
    let lock = zrail_rust::build_lock(&root, "zrail.toml".as_ref()).expect("build lock");
    lock.write(&root.join("zrail.lock")).expect("write lock");
    commit_all(&root);
    root
}

fn contract(root: &str) -> String {
    format!("{CONTRACT_PREFIX}roots = [\"{root}\"]\n{CONTRACT_SUFFIX}")
}

fn discovery_options(root: &std::path::Path) -> MigrateLockOptions {
    MigrateLockOptions {
        root: root.into(),
        config: "zrail.toml".into(),
        lock: "zrail.lock".into(),
        base: "HEAD".into(),
        target: None,
        output: None,
        discover_base: true,
    }
}
