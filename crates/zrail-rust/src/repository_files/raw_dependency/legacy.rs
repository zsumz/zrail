//! Unchanged raw predicates execute only in trusted frozen qualification.

use std::{collections::BTreeSet, fs, path::Path};

use serde::Deserialize;
use zrail_core::{RepositoryFilePredicate, RepositoryFileRule};

#[derive(Debug, Deserialize)]
pub(crate) struct Guardrails {
    schema: u64,
    dependencies: Dependencies,
}

#[derive(Debug, Deserialize)]
struct Dependencies {
    banned: Vec<String>,
}

pub(super) fn check(root: &Path, policy: &RepositoryFileRule) {
    if matches!(policy.predicate, RepositoryFilePredicate::BytesEqual { .. }) {
        read(&root.join(&policy.include[0]));
    } else if policy.name.starts_with("kd-lock-ban-") {
        lock(root);
    } else if policy.name == "kd-dep-ci-gate-count" {
        gate(root);
    } else if policy.name.starts_with("kd-dep-ci-") {
        matrix(root);
    } else {
        attributes(root);
    }
}

fn lockfile_packages(lockfile: &str) -> BTreeSet<&str> {
    lockfile
        .lines()
        .filter_map(|line| {
            line.trim()
                .strip_prefix("name = \"")
                .and_then(|name| name.strip_suffix('"'))
        })
        .collect()
}

fn lock(root: &Path) {
    let workspace_root = || root.to_path_buf();
    let root = workspace_root();
    let guardrails = load_guardrails(&root);
    let lockfile = read(&root.join("Cargo.lock"));
    let packages = lockfile_packages(&lockfile);
    let violations = guardrails
        .dependencies
        .banned
        .iter()
        .filter(|name| packages.contains(name.as_str()))
        .collect::<Vec<_>>();

    assert!(
        violations.is_empty(),
        "forbidden packages in Cargo.lock: {violations:?}"
    );
}

fn gate(root: &Path) {
    let workspace_root = || root.to_path_buf();
    let root = workspace_root();
    let workflow = read(&root.join(".github/workflows/ci.yml"));

    assert_eq!(
        workflow.matches("run: scripts/check").count(),
        1,
        "CI must execute scripts/check exactly once instead of duplicating its policy"
    );
}

fn matrix(root: &Path) {
    let workspace_root = || root.to_path_buf();
    let root = workspace_root();
    let workflow = read(&root.join(".github/workflows/ci.yml"));

    assert!(workflow.contains("os: [ubuntu-latest, macos-latest, windows-latest]"));
    assert!(workflow.contains("runs-on: ${{ matrix.os }}"));
}

fn attributes(root: &Path) {
    let workspace_root = || root.to_path_buf();
    let root = workspace_root();
    let attributes = read(&root.join(".gitattributes"));

    for expected in [
        "*.md text eol=lf",
        "*.rs text eol=lf",
        "*.svg text eol=lf",
        "scripts/* text eol=lf",
    ] {
        assert!(attributes.lines().any(|line| line.trim() == expected));
    }
}

pub(crate) fn load_guardrails(root: &Path) -> Guardrails {
    let source = read(&root.join("guardrails.toml"));
    let config = toml::from_str::<Guardrails>(&source)
        .unwrap_or_else(|error| panic!("parse guardrails.toml: {error}"));
    assert_eq!(config.schema, 1, "unsupported guardrails.toml schema");
    config
}

pub(crate) fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}
