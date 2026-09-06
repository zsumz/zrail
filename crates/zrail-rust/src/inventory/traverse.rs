//! One bounded physical traversal shared by source discovery and repository-file assertions.

use super::{
    RepositoryEntry, RepositoryEntryKind, exclusions::excluded_subtree,
    scan::RepositoryInventoryError,
};
use std::{
    fs,
    path::{Path, PathBuf},
};
use zrail_core::{MAX_DIRECTORY_DEPTH, MAX_REPOSITORY_ENTRIES, repository_relative};

pub(crate) fn scan_file_entries(
    root: &Path,
) -> Result<Vec<RepositoryEntry>, RepositoryInventoryError> {
    scan_repository(root, &[], true).map(|(_, entries)| entries)
}

pub(super) fn scan_repository(
    root: &Path,
    exclusions: &[String],
    include_special: bool,
) -> Result<(PathBuf, Vec<RepositoryEntry>), RepositoryInventoryError> {
    let root = fs::canonicalize(root).map_err(|error| {
        RepositoryInventoryError(format!("open repository {}: {error}", root.display()))
    })?;
    let mut entries = Vec::new();
    collect(&root, &root, exclusions, include_special, &mut entries, 0)?;
    entries.sort_by(|left, right| left.relative.cmp(&right.relative));
    Ok((root, entries))
}

fn collect(
    root: &Path,
    current: &Path,
    exclusions: &[String],
    include_special: bool,
    entries: &mut Vec<RepositoryEntry>,
    depth: usize,
) -> Result<(), RepositoryInventoryError> {
    if depth > MAX_DIRECTORY_DEPTH {
        return Err(RepositoryInventoryError(format!(
            "repository exceeds the {MAX_DIRECTORY_DEPTH}-directory-depth safety limit at {}",
            current.display()
        )));
    }
    let directory = fs::read_dir(current).map_err(|error| {
        RepositoryInventoryError(format!("read {}: {error}", current.display()))
    })?;
    let mut children = Vec::new();
    for child in directory {
        if entries.len() + children.len() == MAX_REPOSITORY_ENTRIES {
            return Err(RepositoryInventoryError(format!(
                "repository exceeds the {MAX_REPOSITORY_ENTRIES}-entry safety limit"
            )));
        }
        children
            .push(child.map_err(|error| RepositoryInventoryError(format!("read entry: {error}")))?);
    }
    children.sort_by_key(fs::DirEntry::file_name);
    for child in children {
        let path = child.path();
        let relative = repository_relative(root, &path).map_err(RepositoryInventoryError)?;
        let metadata = fs::symlink_metadata(&path).map_err(|error| {
            RepositoryInventoryError(format!("inspect {}: {error}", path.display()))
        })?;
        if metadata.file_type().is_symlink() {
            entries.push(RepositoryEntry {
                relative,
                absolute: path,
                kind: RepositoryEntryKind::Symlink,
            });
        } else if metadata.is_dir() {
            entries.push(RepositoryEntry {
                relative: relative.clone(),
                absolute: path.clone(),
                kind: RepositoryEntryKind::Directory,
            });
            if !skip_directory(&relative) && !excluded_subtree(exclusions, &relative) {
                collect(root, &path, exclusions, include_special, entries, depth + 1)?;
            }
        } else if metadata.is_file() {
            entries.push(RepositoryEntry {
                relative,
                absolute: path,
                kind: RepositoryEntryKind::File,
            });
        } else if include_special {
            entries.push(RepositoryEntry {
                relative,
                absolute: path,
                kind: RepositoryEntryKind::Other,
            });
        }
    }
    Ok(())
}

pub(crate) fn skip_directory(relative: &str) -> bool {
    relative == ".git"
        || relative.ends_with("/.git")
        || relative == ".zrail"
        || relative == "target"
}
