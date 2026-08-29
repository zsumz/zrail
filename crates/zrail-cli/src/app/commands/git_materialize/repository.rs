//! Full-repository snapshots retain bounded Git blobs, symlinks, and gitlinks.

use std::{collections::BTreeMap, ffi::OsString, fs, path::Path};

use crate::app::error::CliError;

use super::{
    TemporaryRoot,
    filesystem::{set_executable, write_new},
};
use crate::app::commands::{git_base::TreeEntry, git_process};

const MAX_GIT_BLOB_BYTES: usize = 64 * 1024 * 1024;
const MAX_GIT_SNAPSHOT_BYTES: usize = 1024 * 1024 * 1024;

pub(super) fn materialize(
    repository: &Path,
    tree: &BTreeMap<String, TreeEntry>,
) -> Result<TemporaryRoot, CliError> {
    let temporary = TemporaryRoot::create()?;
    let mut total = 0_usize;
    let mut symlinks = Vec::new();
    for (path, entry) in tree {
        validate_entry(path, entry, &mut total)?;
        if entry.is_gitlink() {
            fs::create_dir_all(temporary.path().join(path)).map_err(|error| {
                CliError::new(format!("create Git snapshot gitlink {path}: {error}"))
            })?;
            continue;
        }
        let bytes = git_process::output(
            repository,
            &[
                OsString::from("cat-file"),
                OsString::from("blob"),
                OsString::from(&entry.object),
            ],
            MAX_GIT_BLOB_BYTES,
            "base repository object read",
        )?;
        if bytes.len() != entry.size {
            return Err(CliError::new(format!(
                "Git base object size changed while reading {path}"
            )));
        }
        if entry.is_symlink() {
            symlinks.push((path.as_str(), bytes));
            continue;
        }
        let destination = temporary.path().join(path);
        write_new(&destination, &bytes)?;
        set_executable(&destination, entry.mode == "100755")?;
    }
    for (path, target) in &symlinks {
        create_internal_symlink(temporary.path(), path, target)?;
    }
    for (path, _) in symlinks {
        validate_internal_symlink(temporary.path(), path)?;
    }
    Ok(temporary)
}

fn validate_entry(path: &str, entry: &TreeEntry, total: &mut usize) -> Result<(), CliError> {
    if !entry.is_regular() && !entry.is_symlink() && !entry.is_gitlink() {
        return Err(CliError::new(format!(
            "full Git base migration does not support Git mode {} at {path}",
            entry.mode
        )));
    }
    if entry.size > MAX_GIT_BLOB_BYTES {
        return Err(CliError::new(format!(
            "Git base blob exceeds the {MAX_GIT_BLOB_BYTES}-byte migration limit: {path}"
        )));
    }
    *total = total
        .checked_add(entry.size)
        .ok_or_else(|| CliError::new("Git base snapshot byte count overflowed"))?;
    if *total > MAX_GIT_SNAPSHOT_BYTES {
        return Err(CliError::new(format!(
            "Git base snapshot exceeds the {MAX_GIT_SNAPSHOT_BYTES}-byte migration limit"
        )));
    }
    Ok(())
}

fn create_internal_symlink(root: &Path, path: &str, target: &[u8]) -> Result<(), CliError> {
    let target = target_path(target)?;
    let destination = root.join(path);
    let parent = destination
        .parent()
        .ok_or_else(|| CliError::new(format!("snapshot symlink has no parent: {path}")))?;
    if target.is_absolute() || !lexically_inside(root, &parent.join(&target)) {
        return Err(CliError::new(format!(
            "Git snapshot symlink escapes the repository: {path}"
        )));
    }
    fs::create_dir_all(parent)
        .map_err(|error| CliError::new(format!("create {}: {error}", parent.display())))?;
    create_symlink(&target, &destination)?;
    Ok(())
}

fn validate_internal_symlink(root: &Path, path: &str) -> Result<(), CliError> {
    let destination = root.join(path);
    let root = fs::canonicalize(root)
        .map_err(|error| CliError::new(format!("resolve Git snapshot root: {error}")))?;
    let resolved = fs::canonicalize(&destination)
        .map_err(|error| CliError::new(format!("resolve Git snapshot symlink {path}: {error}")))?;
    if !resolved.starts_with(root) {
        return Err(CliError::new(format!(
            "Git snapshot symlink escapes the repository: {path}"
        )));
    }
    Ok(())
}

fn lexically_inside(root: &Path, candidate: &Path) -> bool {
    let Ok(relative) = candidate.strip_prefix(root) else {
        return false;
    };
    let mut depth = 0_usize;
    for component in relative.components() {
        match component {
            std::path::Component::Normal(_) => depth += 1,
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir if depth > 0 => depth -= 1,
            std::path::Component::ParentDir
            | std::path::Component::RootDir
            | std::path::Component::Prefix(_) => return false,
        }
    }
    true
}

#[cfg(unix)]
fn target_path(bytes: &[u8]) -> Result<std::path::PathBuf, CliError> {
    use std::os::unix::ffi::OsStrExt as _;
    if bytes.contains(&0) {
        return Err(CliError::new(
            "Git snapshot symlink target contains a NUL byte",
        ));
    }
    Ok(Path::new(std::ffi::OsStr::from_bytes(bytes)).to_path_buf())
}

#[cfg(not(unix))]
fn target_path(bytes: &[u8]) -> Result<std::path::PathBuf, CliError> {
    let value = std::str::from_utf8(bytes)
        .map_err(|_| CliError::new("Git snapshot symlink target is not UTF-8"))?;
    Ok(value.into())
}

fn create_symlink(target: &Path, destination: &Path) -> Result<(), CliError> {
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target, destination).map_err(|error| {
            CliError::new(format!(
                "create Git snapshot symlink {}: {error}",
                destination.display()
            ))
        })
    }
    #[cfg(windows)]
    {
        let parent = destination.parent().ok_or_else(|| {
            CliError::new(format!(
                "snapshot symlink has no parent: {}",
                destination.display()
            ))
        })?;
        let result = if parent.join(target).is_dir() {
            std::os::windows::fs::symlink_dir(target, destination)
        } else {
            std::os::windows::fs::symlink_file(target, destination)
        };
        result.map_err(|error| {
            CliError::new(format!(
                "create Git snapshot symlink {}: {error}",
                destination.display()
            ))
        })
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = (target, destination);
        Err(CliError::new(
            "Git snapshot symlinks are unsupported on this platform",
        ))
    }
}
