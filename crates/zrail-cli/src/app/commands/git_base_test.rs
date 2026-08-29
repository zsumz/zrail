//! Git snapshots read committed architecture inputs without copying repository source.

use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

use super::GitSnapshot;

use crate::app::{
    args::{DiffMode, DiffOptions},
    output::OutputFormat,
};

pub(crate) static FIXTURE_SEQUENCE: AtomicUsize = AtomicUsize::new(0);

pub(crate) const CONTRACT_PREFIX: &str = r#"schema = 1
adapters = ["rust"]

[repository]
"#;

pub(crate) const CONTRACT_SUFFIX: &str = r#"exclude = []
workspace_members = "exact"
nested_git = "deny"
submodules = "deny"
symlinks = "inside"

[dependencies]
mode = "locked"
unassigned_packages = "allow"
cycles = "deny"

[source.rust]
module_docs = "allow"
facades = "allow"
tests = "allow"

[source.rust.hygiene]
unsafe = "allow"
lint_suppressions = "allow"
deny_methods = []
deny_macros = []

[source.rust.size.facade]
target = 300
hard = 300

[source.rust.size.implementation]
target = 300
hard = 300

[source.rust.size.test]
target = 300
hard = 300

[source.rust.size.auxiliary]
target = 300
hard = 300
"#;

#[test]
fn base_diff_compares_the_commit_with_the_current_worktree() {
    if !git_available() {
        return;
    }
    let root = fixture_root("diff");
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create fixture");
    fs::write(
        root.join("zrail.toml"),
        format!("{CONTRACT_PREFIX}roots = [\".\"]\n{CONTRACT_SUFFIX}"),
    )
    .expect("write base contract");
    commit_all(&root);
    fs::write(
        root.join("zrail.toml"),
        format!("{CONTRACT_PREFIX}roots = [\"src\"]\n{CONTRACT_SUFFIX}"),
    )
    .expect("narrow current contract");

    let output = crate::app::commands::diff(&DiffOptions {
        mode: DiffMode::Base {
            root: root.clone(),
            revision: "HEAD".into(),
        },
        config: "zrail.toml".into(),
        lock: "zrail.lock".into(),
        format: OutputFormat::Human,
        deny_grants: true,
    })
    .expect("compare Git base");

    assert_eq!(output.exit_code, 1);
    assert!(output.text.contains("GRANT repository.root"));
    reset(&root);
}

#[test]
fn snapshot_materializes_exact_and_wildcard_contract_inputs() {
    if !git_available() {
        return;
    }
    let root = fixture_root("imports");
    reset(&root);
    fs::create_dir_all(root.join("architecture")).expect("create architecture directory");
    fs::write(
        root.join("zrail.toml"),
        "schema = 1\nadapters = ['rust']\nimports = ['architecture/*.toml']\n",
    )
    .expect("write root contract");
    fs::write(
        root.join("architecture/base.toml"),
        "[repository]\nroots = ['.']\n",
    )
    .expect("write fragment");
    fs::write(root.join("zrail.lock"), "base lock\n").expect("write lock");
    fs::write(root.join("private-source.rs"), "secret\n").expect("write unrelated source");
    commit_all(&root);

    let snapshot = GitSnapshot::create(
        &root,
        OsStr::new("HEAD"),
        Path::new("zrail.toml"),
        Path::new("zrail.lock"),
    )
    .expect("materialize Git base");

    assert_eq!(
        fs::read_to_string(snapshot.root().join("architecture/base.toml")).expect("read fragment"),
        "[repository]\nroots = ['.']\n"
    );
    assert_eq!(
        fs::read_to_string(snapshot.root().join("zrail.lock")).expect("read lock"),
        "base lock\n"
    );
    assert!(!snapshot.root().join("private-source.rs").exists());
    reset(&root);
}

#[cfg(unix)]
#[test]
fn snapshot_rejects_symlinked_architecture_inputs() {
    use std::os::unix::fs::symlink;

    if !git_available() {
        return;
    }
    let root = fixture_root("symlink");
    reset(&root);
    fs::create_dir_all(&root).expect("create fixture");
    fs::write(root.join("actual.toml"), "schema = 1\n").expect("write target");
    symlink("actual.toml", root.join("zrail.toml")).expect("create symlink");
    commit_all(&root);

    let error = GitSnapshot::create(
        &root,
        OsStr::new("HEAD"),
        Path::new("zrail.toml"),
        Path::new("zrail.lock"),
    )
    .expect_err("architecture symlinks must fail closed");

    assert!(error.message.contains("not a regular file"));
    reset(&root);
}

pub(crate) fn commit_all(root: &Path) {
    run_git(root, &["init", "--quiet"]);
    run_git(root, &["add", "."]);
    commit_index(root);
}

pub(crate) fn commit_index(root: &Path) {
    run_git(
        root,
        &[
            "-c",
            "user.name=zrail tests",
            "-c",
            "user.email=zrail@example.invalid",
            "-c",
            "commit.gpgsign=false",
            "commit",
            "--quiet",
            "-m",
            "fixture",
        ],
    );
}

pub(crate) fn head_revision(root: &Path) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "HEAD"])
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .expect("resolve fixture revision");
    assert!(output.status.success(), "resolve fixture revision");
    String::from_utf8(output.stdout)
        .expect("UTF-8 revision")
        .trim()
        .into()
}

pub(crate) fn run_git(root: &Path, arguments: &[&str]) {
    let status = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(arguments)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_TERMINAL_PROMPT", "0")
        .status()
        .expect("run Git fixture command");
    assert!(
        status.success(),
        "Git fixture command failed: {arguments:?}"
    );
}

pub(crate) fn git_available() -> bool {
    Command::new("git")
        .arg("--version")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

pub(crate) fn fixture_root(name: &str) -> PathBuf {
    let sequence = FIXTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "zrail-git-base-{}-{sequence}-{name}",
        std::process::id()
    ))
}

pub(crate) fn reset(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).expect("reset fixture");
    }
}
