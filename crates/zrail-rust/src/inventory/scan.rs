//! Deterministic filesystem traversal bounded to the repository root.

use std::{
    error::Error,
    fmt, fs,
    path::{Path, PathBuf},
};

use zrail_core::{
    Contract, MAX_DIRECTORY_DEPTH, MAX_REPOSITORY_ENTRIES, read_text_with_limit,
    repository_relative,
};

use super::{
    classify::{classify_path, is_indexed_source},
    exclusions::{excluded, excluded_subtree},
    types::{RepositoryEntry, RepositoryEntryKind, RepositoryInventory, RustSourceFile},
};

const MAX_CARGO_MANIFESTS: usize = 10_000;
const MAX_RUST_FILES: usize = 20_000;
const MAX_RUST_SOURCE_BYTES: usize = 2 * 1024 * 1024;
const MAX_TOTAL_RUST_SOURCE_BYTES: usize = 128 * 1024 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RepositoryInventoryError(String);

impl fmt::Display for RepositoryInventoryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for RepositoryInventoryError {}

pub(crate) fn inventory_repository(
    root: &Path,
    contract: &Contract,
) -> Result<RepositoryInventory, RepositoryInventoryError> {
    let (root, entries) = scan_repository(root, &contract.repository.exclude)?;
    let mut rust_files = Vec::new();
    let manifests = cargo_manifests(&entries, Some(contract))?;
    let mut source_bytes = 0_usize;
    for entry in &entries {
        if entry.kind != RepositoryEntryKind::File || excluded(contract, &entry.relative) {
            continue;
        }
        if is_indexed_source(&entry.relative, &contract.source.rust.generated)
            && under_roots(contract, &entry.relative)
        {
            if rust_files.len() == MAX_RUST_FILES {
                return Err(RepositoryInventoryError(format!(
                    "repository exceeds the {MAX_RUST_FILES}-Rust-file safety limit"
                )));
            }
            let source = read_text_with_limit(&entry.absolute, MAX_RUST_SOURCE_BYTES)
                .map_err(RepositoryInventoryError)?;
            source_bytes = add_source_bytes(source_bytes, source.len())?;
            rust_files.push(RustSourceFile {
                relative: entry.relative.clone(),
                class: classify_path(&entry.relative, &contract.source.rust.generated),
                lines: source.lines().count(),
                source,
            });
        }
    }
    rust_files.sort_by(|left, right| left.relative.cmp(&right.relative));
    Ok(RepositoryInventory {
        root,
        entries,
        rust_files,
        manifest_paths: manifests,
    })
}

fn add_source_bytes(current: usize, observed: usize) -> Result<usize, RepositoryInventoryError> {
    let total = current
        .checked_add(observed)
        .ok_or_else(|| RepositoryInventoryError("Rust source byte count overflowed".into()))?;
    if total > MAX_TOTAL_RUST_SOURCE_BYTES {
        return Err(RepositoryInventoryError(format!(
            "repository exceeds the {MAX_TOTAL_RUST_SOURCE_BYTES}-byte total Rust source safety limit"
        )));
    }
    Ok(total)
}

pub(crate) fn inventory_cargo_repository(
    root: &Path,
) -> Result<RepositoryInventory, RepositoryInventoryError> {
    let (root, entries) = scan_repository(root, &[])?;
    let manifest_paths = cargo_manifests(&entries, None)?;
    Ok(RepositoryInventory {
        root,
        entries,
        rust_files: Vec::new(),
        manifest_paths,
    })
}

fn scan_repository(
    root: &Path,
    exclusions: &[String],
) -> Result<(PathBuf, Vec<RepositoryEntry>), RepositoryInventoryError> {
    let root = fs::canonicalize(root).map_err(|error| {
        RepositoryInventoryError(format!("open repository {}: {error}", root.display()))
    })?;
    let mut entries = Vec::new();
    collect(&root, &root, exclusions, &mut entries, 0)?;
    entries.sort_by(|left, right| left.relative.cmp(&right.relative));
    Ok((root, entries))
}

fn cargo_manifests(
    entries: &[RepositoryEntry],
    contract: Option<&Contract>,
) -> Result<Vec<PathBuf>, RepositoryInventoryError> {
    let mut manifests = Vec::new();
    for entry in entries {
        let excluded = contract.is_some_and(|contract| excluded(contract, &entry.relative));
        if entry.kind != RepositoryEntryKind::File
            || excluded
            || Path::new(&entry.relative)
                .file_name()
                .is_none_or(|name| name != "Cargo.toml")
        {
            continue;
        }
        if manifests.len() == MAX_CARGO_MANIFESTS {
            return Err(RepositoryInventoryError(format!(
                "repository exceeds the {MAX_CARGO_MANIFESTS}-manifest safety limit"
            )));
        }
        manifests.push(entry.absolute.clone());
    }
    manifests.sort();
    Ok(manifests)
}

fn collect(
    root: &Path,
    current: &Path,
    exclusions: &[String],
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
                collect(root, &path, exclusions, entries, depth + 1)?;
            }
        } else if metadata.is_file() {
            entries.push(RepositoryEntry {
                relative,
                absolute: path,
                kind: RepositoryEntryKind::File,
            });
        }
    }
    Ok(())
}

fn skip_directory(relative: &str) -> bool {
    relative == ".git"
        || relative.ends_with("/.git")
        || relative == ".zrail"
        || relative == "target"
}

fn under_roots(contract: &Contract, relative: &str) -> bool {
    contract
        .repository
        .roots
        .iter()
        .any(|root| root == "." || relative == root || relative.starts_with(&format!("{root}/")))
}

#[cfg(test)]
#[path = "scan_test.rs"]
mod scan_test;
