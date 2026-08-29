//! Exact ancestry and worktree checks for migration acceptance.

use std::{ffi::OsString, fs, path::Path};

use zrail_core::{MAX_INPUT_BYTES, normalize_relative, read_bytes_with_limit, repository_file};

use crate::app::error::CliError;

use super::super::{
    git_base::{GitSnapshot, TreeEntry},
    git_process,
};

const MAX_CHANGED_PATH_BYTES: usize = 64 * 1024 * 1024;

pub(in crate::app::commands) fn require_ancestor(
    repository: &Path,
    base_commit: &str,
    target_commit: &str,
) -> Result<(), CliError> {
    let output = git_process::output(
        repository,
        &[
            OsString::from("merge-base"),
            OsString::from(base_commit),
            OsString::from(target_commit),
        ],
        128,
        "migration ancestry check",
    )?;
    if trim_ascii(&output) == base_commit.as_bytes() {
        Ok(())
    } else {
        Err(CliError::new(
            "migration target must be the base revision or its descendant",
        ))
    }
}

pub(in crate::app::commands) fn require_worktree_target(
    repository: &Path,
    target: &GitSnapshot,
    report_path: &Path,
    report: &str,
) -> Result<(), CliError> {
    let report_relative = normalize_relative(report_path).map_err(CliError::new)?;
    if report_relative.is_empty() || target.tree.contains_key(&report_relative) {
        return Err(CliError::new(
            "migration report must be an untracked repository file",
        ));
    }
    let report_file = repository_file(repository, report_path).map_err(CliError::new)?;
    let observed_report = read_bytes_with_limit(&report_file, MAX_INPUT_BYTES)
        .map_err(|error| CliError::new(format!("read migration report: {error}")))?;
    if observed_report != report.as_bytes() {
        return Err(CliError::new(
            "migration report does not match the reviewed bridge",
        ));
    }
    require_clean_index(repository, target)?;
    require_only_report_untracked(repository, &report_relative)?;
    for (path, entry) in &target.tree {
        compare_entry(repository, target, path, entry)?;
    }
    Ok(())
}

fn require_clean_index(repository: &Path, target: &GitSnapshot) -> Result<(), CliError> {
    let staged = git_process::output(
        repository,
        &[
            OsString::from("diff"),
            OsString::from("--cached"),
            OsString::from("--name-only"),
            OsString::from("--no-renames"),
            OsString::from("-z"),
            OsString::from(target.commit()),
            OsString::from("--"),
        ],
        MAX_CHANGED_PATH_BYTES,
        "migration target index check",
    )?;
    if staged.is_empty() {
        Ok(())
    } else {
        target_mismatch()
    }
}

fn require_only_report_untracked(repository: &Path, report: &str) -> Result<(), CliError> {
    let untracked = git_process::output(
        repository,
        &[
            OsString::from("ls-files"),
            OsString::from("--others"),
            OsString::from("--exclude-standard"),
            OsString::from("-z"),
        ],
        MAX_CHANGED_PATH_BYTES,
        "migration target untracked-file check",
    )?;
    for path in nul_paths(&untracked)? {
        if path != report {
            return target_mismatch();
        }
    }
    Ok(())
}

fn compare_entry(
    repository: &Path,
    target: &GitSnapshot,
    path: &str,
    entry: &TreeEntry,
) -> Result<(), CliError> {
    let actual = repository.join(path);
    let expected = target.root().join(path);
    if entry.is_regular() {
        compare_regular(&actual, &expected, entry)
    } else if entry.is_symlink() {
        compare_symlink(&actual, &expected)
    } else if entry.is_gitlink() {
        compare_gitlink(&actual, &entry.object)
    } else {
        target_mismatch()
    }
}

fn compare_regular(actual: &Path, expected: &Path, entry: &TreeEntry) -> Result<(), CliError> {
    let metadata = fs::symlink_metadata(actual).map_err(|_| mismatch_error())?;
    if !metadata.is_file()
        || metadata.file_type().is_symlink()
        || metadata.len() != entry.size as u64
    {
        return target_mismatch();
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        if (metadata.permissions().mode() & 0o111 != 0) != (entry.mode == "100755") {
            return target_mismatch();
        }
    }
    let actual = read_bytes_with_limit(actual, entry.size).map_err(|_| mismatch_error())?;
    let expected = read_bytes_with_limit(expected, entry.size).map_err(CliError::new)?;
    if actual == expected {
        Ok(())
    } else {
        target_mismatch()
    }
}

fn compare_symlink(actual: &Path, expected: &Path) -> Result<(), CliError> {
    let metadata = fs::symlink_metadata(actual).map_err(|_| mismatch_error())?;
    if !metadata.file_type().is_symlink() {
        return target_mismatch();
    }
    let actual = fs::read_link(actual).map_err(|_| mismatch_error())?;
    let expected = fs::read_link(expected)
        .map_err(|error| CliError::new(format!("read migration target symlink: {error}")))?;
    if actual == expected {
        Ok(())
    } else {
        target_mismatch()
    }
}

fn compare_gitlink(actual: &Path, object: &str) -> Result<(), CliError> {
    let metadata = match fs::symlink_metadata(actual) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(_) => return target_mismatch(),
    };
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return target_mismatch();
    }
    if !actual.join(".git").exists() {
        return Ok(());
    }
    let commit = git_process::output(
        actual,
        &[
            OsString::from("rev-parse"),
            OsString::from("--verify"),
            OsString::from("HEAD^{commit}"),
        ],
        128,
        "migration target submodule check",
    )?;
    if trim_ascii(&commit) == object.as_bytes() {
        Ok(())
    } else {
        target_mismatch()
    }
}

fn nul_paths(bytes: &[u8]) -> Result<Vec<String>, CliError> {
    bytes
        .split(|byte| *byte == 0)
        .filter(|path| !path.is_empty())
        .map(|path| {
            let path = std::str::from_utf8(path)
                .map_err(|_| CliError::new("Git returned a non-UTF-8 worktree path"))?;
            let normalized = normalize_relative(Path::new(path)).map_err(CliError::new)?;
            if normalized != path {
                return Err(CliError::new("Git returned a noncanonical worktree path"));
            }
            Ok(normalized)
        })
        .collect()
}

fn target_mismatch<T>() -> Result<T, CliError> {
    Err(mismatch_error())
}

fn mismatch_error() -> CliError {
    CliError::new("current worktree does not match the reviewed migration target commit")
}

fn trim_ascii(mut bytes: &[u8]) -> &[u8] {
    while bytes.last().is_some_and(u8::is_ascii_whitespace) {
        bytes = &bytes[..bytes.len() - 1];
    }
    bytes
}
