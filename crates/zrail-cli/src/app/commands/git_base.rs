//! Exact architecture inputs materialized from an explicitly requested Git commit.

use std::{
    collections::BTreeMap,
    ffi::{OsStr, OsString},
    fs,
    path::Path,
};

use zrail_core::{MAX_INPUT_BYTES, MAX_REPOSITORY_ENTRIES, normalize_relative};

use crate::app::error::CliError;

use super::{git_materialize, git_process};

const MAX_GIT_TREE_BYTES: usize = 64 * 1024 * 1024;

#[derive(Debug)]
pub(super) struct GitSnapshot {
    temporary: git_materialize::TemporaryRoot,
    commit: String,
}

impl GitSnapshot {
    pub(super) fn create(
        repository: &Path,
        revision: &OsStr,
        config: &Path,
        lock: &Path,
    ) -> Result<Self, CliError> {
        let repository = fs::canonicalize(repository).map_err(|error| {
            CliError::new(format!(
                "open Git repository {}: {error}",
                repository.display()
            ))
        })?;
        require_top_level(&repository)?;
        let commit = resolve_commit(&repository, revision)?;
        let tree = read_tree(&repository, &commit)?;
        let temporary = git_materialize::materialize(&repository, &tree, config, lock)?;
        Ok(Self { temporary, commit })
    }

    pub(super) fn create_repository(repository: &Path, revision: &OsStr) -> Result<Self, CliError> {
        let repository = fs::canonicalize(repository).map_err(|error| {
            CliError::new(format!(
                "open Git repository {}: {error}",
                repository.display()
            ))
        })?;
        require_top_level(&repository)?;
        let commit = resolve_commit(&repository, revision)?;
        let tree = read_tree(&repository, &commit)?;
        let temporary = git_materialize::materialize_repository(&repository, &tree)?;
        Ok(Self { temporary, commit })
    }

    pub(super) fn root(&self) -> &Path {
        self.temporary.path()
    }

    pub(super) fn commit(&self) -> &str {
        &self.commit
    }
}

#[derive(Clone, Debug)]
pub(super) struct TreeEntry {
    pub(super) mode: String,
    pub(super) object: String,
    pub(super) size: usize,
}

impl TreeEntry {
    pub(super) fn is_regular(&self) -> bool {
        matches!(self.mode.as_str(), "100644" | "100755")
    }
}

fn require_top_level(repository: &Path) -> Result<(), CliError> {
    let output = git_process::output(
        repository,
        &[OsString::from("rev-parse"), OsString::from("--show-prefix")],
        MAX_INPUT_BYTES,
        "repository discovery",
    )?;
    if trim_ascii(&output).is_empty() {
        Ok(())
    } else {
        Err(CliError::new(format!(
            "--root must name the Git worktree root: {}",
            repository.display()
        )))
    }
}

fn resolve_commit(repository: &Path, revision: &OsStr) -> Result<String, CliError> {
    let mut commit_expression = revision.to_os_string();
    commit_expression.push("^{commit}");
    let output = git_process::output(
        repository,
        &[
            OsString::from("rev-parse"),
            OsString::from("--verify"),
            OsString::from("--end-of-options"),
            commit_expression,
        ],
        128,
        "base revision resolution",
    )?;
    let commit = std::str::from_utf8(trim_ascii(&output))
        .map_err(|_| CliError::new("Git returned a non-UTF-8 commit identifier"))?;
    if !matches!(commit.len(), 40 | 64) || !commit.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(CliError::new("Git returned an invalid commit identifier"));
    }
    Ok(commit.to_owned())
}

fn read_tree(repository: &Path, commit: &str) -> Result<BTreeMap<String, TreeEntry>, CliError> {
    let output = git_process::output(
        repository,
        &[
            OsString::from("ls-tree"),
            OsString::from("-r"),
            OsString::from("-z"),
            OsString::from("-l"),
            OsString::from("--full-tree"),
            OsString::from(commit),
        ],
        MAX_GIT_TREE_BYTES,
        "base tree inventory",
    )?;
    let mut tree = BTreeMap::new();
    for record in output
        .split(|byte| *byte == 0)
        .filter(|record| !record.is_empty())
    {
        if tree.len() == MAX_REPOSITORY_ENTRIES {
            return Err(CliError::new(format!(
                "Git base tree exceeds the {MAX_REPOSITORY_ENTRIES}-entry safety limit"
            )));
        }
        let (header, path) = split_once(record, b'\t')
            .ok_or_else(|| CliError::new("Git returned a malformed tree entry"))?;
        let header = std::str::from_utf8(header)
            .map_err(|_| CliError::new("Git returned non-UTF-8 tree metadata"))?;
        let path = std::str::from_utf8(path)
            .map_err(|_| CliError::new("Git base tree contains a non-UTF-8 path"))?;
        let normalized = normalize_relative(Path::new(path)).map_err(CliError::new)?;
        if normalized.is_empty() || normalized != path {
            return Err(CliError::new(format!(
                "Git base tree contains a noncanonical path {path:?}"
            )));
        }
        let fields = header.split_ascii_whitespace().collect::<Vec<_>>();
        if fields.len() != 4 {
            return Err(CliError::new("Git returned malformed tree metadata"));
        }
        if !matches!(fields[2].len(), 40 | 64)
            || !fields[2].bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            return Err(CliError::new("Git returned an invalid object identifier"));
        }
        if fields[1] == "commit" && fields[0] == "160000" {
            if tree
                .insert(
                    normalized,
                    TreeEntry {
                        mode: fields[0].to_owned(),
                        object: fields[2].to_owned(),
                        size: 0,
                    },
                )
                .is_some()
            {
                return Err(CliError::new("Git returned a duplicate tree path"));
            }
            continue;
        }
        if fields[1] != "blob" {
            continue;
        }
        let size = fields[3]
            .parse::<usize>()
            .map_err(|_| CliError::new("Git returned an invalid blob size"))?;
        let entry = TreeEntry {
            mode: fields[0].to_owned(),
            object: fields[2].to_owned(),
            size,
        };
        if tree.insert(normalized, entry).is_some() {
            return Err(CliError::new("Git returned a duplicate tree path"));
        }
    }
    Ok(tree)
}

fn split_once(bytes: &[u8], separator: u8) -> Option<(&[u8], &[u8])> {
    let index = bytes.iter().position(|byte| *byte == separator)?;
    Some((&bytes[..index], &bytes[index + 1..]))
}

fn trim_ascii(mut bytes: &[u8]) -> &[u8] {
    while bytes.last().is_some_and(u8::is_ascii_whitespace) {
        bytes = &bytes[..bytes.len() - 1];
    }
    bytes
}

#[cfg(test)]
#[path = "git_base_test.rs"]
mod git_base_test;

#[cfg(test)]
pub(super) use git_base_test::{
    CONTRACT_PREFIX, CONTRACT_SUFFIX, commit_all, fixture_root, git_available, reset,
};
