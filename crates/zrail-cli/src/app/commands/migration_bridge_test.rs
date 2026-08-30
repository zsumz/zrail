//! Cross-revision migration binds source repairs without absorbing policy grants.

use std::{fs, path::PathBuf};

use crate::app::{
    args::{CommonOptions, MigrateLockOptions, UpdateOptions},
    output::OutputFormat,
};

use super::super::{
    git_base::{
        CONTRACT_PREFIX, CONTRACT_SUFFIX, commit_all, commit_index, fixture_root, git_available,
        head_revision, reset, run_git,
    },
    migrate_lock::migrate_lock,
    update::update,
};

#[test]
fn descendant_target_bridges_a_base_the_current_engine_cannot_analyze() {
    if !git_available() {
        return;
    }
    let (root, base) = bridge_fixture("descendant", false);
    let strict = migrate_lock(&migration_options(&root, &base, None))
        .expect_err("same-revision migration must preserve its strict path");
    assert!(strict.message.contains("reanalyze migration base"));

    let output = migrate_lock(&migration_options(&root, &base, Some("HEAD")))
        .expect("bridge descendant revision");
    let artifact = fs::read_to_string(root.join("migration.json")).expect("read bridge");

    assert!(artifact.contains("\"base_analysis_error\""));
    assert!(artifact.contains("\"path\": \"src/lib.rs\""));
    assert!(artifact.contains("\"lock_sha256\""));
    let accepted = update(&update_options(&root, &base, Some(acceptance(&output))))
        .expect("accept reviewed bridge");
    assert_eq!(accepted.exit_code, 0);
    reset(&root);
}

#[test]
fn target_source_mutation_invalidates_the_reviewed_bridge() {
    if !git_available() {
        return;
    }
    let (root, base) = bridge_fixture("mutation", false);
    let report =
        migrate_lock(&migration_options(&root, &base, Some("HEAD"))).expect("build bridge");
    fs::write(
        root.join("src/lib.rs"),
        "//! fixed\n// changed after review\n",
    )
    .expect("mutate target source");

    let refused = update(&update_options(&root, &base, Some(acceptance(&report))))
        .expect("refuse changed target");

    assert_eq!(refused.exit_code, 1);
    assert!(
        refused
            .text
            .contains("does not match the reviewed migration target")
    );
    reset(&root);
}

#[test]
fn untracked_and_index_hidden_changes_invalidate_the_reviewed_target() {
    if !git_available() {
        return;
    }
    for flag in [None, Some("--assume-unchanged"), Some("--skip-worktree")] {
        let (root, base) = bridge_fixture(flag.unwrap_or("untracked"), false);
        let report = migrate_lock(&migration_options(&root, &base, Some("HEAD")))
            .expect("build reviewed bridge");
        if let Some(flag) = flag {
            run_git(&root, &["update-index", flag, "src/lib.rs"]);
            fs::write(root.join("src/lib.rs"), "//! fixed\n// hidden edit\n")
                .expect("write hidden edit");
        } else {
            fs::write(root.join("review-notes.txt"), "untracked\n").expect("write untracked file");
        }

        let refused = update(&update_options(&root, &base, Some(acceptance(&report))))
            .expect("refuse non-target worktree");
        assert_eq!(refused.exit_code, 1);
        assert!(refused.text.contains("does not match the reviewed"));
        reset(&root);
    }
}

#[test]
fn target_must_retain_the_exact_prior_lock() {
    if !git_available() {
        return;
    }
    for remove in [false, true] {
        let (root, base) = bridge_fixture(
            if remove {
                "removed-lock"
            } else {
                "changed-lock"
            },
            false,
        );
        let lock = root.join("zrail.lock");
        if remove {
            fs::remove_file(&lock).expect("remove target lock");
        } else {
            let mut bytes = fs::read(&lock).expect("read target lock");
            bytes.push(b'\n');
            fs::write(&lock, bytes).expect("change target lock");
        }
        commit_all(&root);

        let error = migrate_lock(&migration_options(&root, &base, Some("HEAD")))
            .expect_err("reject changed target lock");
        assert!(error.message.contains("retain the exact base lock"));
        reset(&root);
    }
}

#[test]
fn migration_report_cannot_replace_a_tracked_target_file() {
    if !git_available() {
        return;
    }
    let (root, base) = bridge_fixture("output-collision", false);
    let mut options = migration_options(&root, &base, Some("HEAD"));
    options.output = "src/lib.rs".into();

    let error = migrate_lock(&options).expect_err("reject tracked report output");
    assert!(error.message.contains("must not replace a tracked target"));
    reset(&root);
}

#[cfg(unix)]
#[test]
fn repository_snapshot_binds_internal_symlinks() {
    use std::os::unix::fs::symlink;

    if !git_available() {
        return;
    }
    let (root, base) = bridge_fixture_root("repository-shapes", false, "src");
    fs::write(root.join("asset.txt"), "asset\n").expect("write asset");
    symlink("asset.txt", root.join("asset-link")).expect("create internal symlink");
    commit_all(&root);

    migrate_lock(&migration_options(&root, &base, Some("HEAD")))
        .expect("snapshot repository shapes");
    let report = fs::read_to_string(root.join("migration.json")).expect("read report");
    assert!(report.contains("\"mode\": \"120000\""));
    reset(&root);
}

#[path = "migration_bridge_test/tests/gitlinks.rs"]
mod gitlinks;

#[test]
fn migration_acceptance_does_not_accept_target_policy_grants() {
    if !git_available() {
        return;
    }
    let (root, base) = bridge_fixture("grant", true);
    let report = migrate_lock(&migration_options(&root, &base, Some("HEAD")))
        .expect("build bridge with contract change");

    let refused = update(&update_options(&root, &base, Some(acceptance(&report))))
        .expect("keep grant authority separate");

    assert_eq!(refused.exit_code, 1);
    assert!(refused.text.contains("GRANT repository.root"));
    reset(&root);
}

fn bridge_fixture(name: &str, weaken_target: bool) -> (PathBuf, String) {
    bridge_fixture_root(name, weaken_target, ".")
}

fn bridge_fixture_root(name: &str, weaken_target: bool, governed_root: &str) -> (PathBuf, String) {
    let root = fixture_root(name);
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create fixture");
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname='fixture'\nversion='0.0.0'\nedition='2024'\n",
    )
    .expect("write manifest");
    fs::write(root.join("src/lib.rs"), "//! original\n").expect("write source");
    fs::write(root.join("zrail.toml"), contract(governed_root)).expect("write contract");
    let mut old = zrail_rust::build_lock(&root, "zrail.toml".as_ref()).expect("build old lock");
    old.schema = 3;
    old.semantics = 4;
    old.analysis
        .as_mut()
        .expect("analysis certificate")
        .analyzer_semantics = 4;
    old.write(&root.join("zrail.lock")).expect("write old lock");
    fs::write(root.join("src/lib.rs"), [0xff]).expect("make base unanalyzable");
    commit_all(&root);
    let base = head_revision(&root);

    fs::write(root.join("src/lib.rs"), "//! fixed\n").expect("repair source");
    if weaken_target {
        fs::write(root.join("zrail.toml"), contract("src")).expect("weaken target contract");
    }
    commit_all(&root);
    (root, base)
}

fn contract(root: &str) -> String {
    format!("{CONTRACT_PREFIX}roots = [\"{root}\"]\n{CONTRACT_SUFFIX}")
}

fn migration_options(
    root: &std::path::Path,
    base: &str,
    target: Option<&str>,
) -> MigrateLockOptions {
    MigrateLockOptions {
        root: root.into(),
        config: "zrail.toml".into(),
        lock: "zrail.lock".into(),
        base: base.into(),
        target: target.map(Into::into),
        output: "migration.json".into(),
    }
}

fn update_options(root: &std::path::Path, base: &str, token: Option<String>) -> UpdateOptions {
    UpdateOptions {
        common: CommonOptions {
            root: root.into(),
            config: "zrail.toml".into(),
            lock: "zrail.lock".into(),
            format: OutputFormat::Human,
            ..CommonOptions::default()
        },
        base: base.into(),
        accept_grants: false,
        accept_migration: token,
        migration_report: Some("migration.json".into()),
    }
}

fn acceptance(output: &super::super::CommandResult) -> String {
    output
        .text
        .lines()
        .find_map(|line| line.strip_prefix("accept with: --accept-migration "))
        .expect("acceptance identity")
        .split_ascii_whitespace()
        .next()
        .expect("acceptance token")
        .into()
}
