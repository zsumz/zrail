//! Deterministic filesystem traversal bounded to the repository root.

#[path = "scan_referenced.rs"]
mod referenced;
pub(crate) use referenced::load_referenced_source;

use std::{
    error::Error,
    fmt,
    path::{Path, PathBuf},
};

use zrail_core::{Contract, read_text_with_limit};

use super::{
    classify::{classify_path, is_indexed_source},
    exclusions::{excluded, excluded_by},
    traverse::scan_repository,
    types::{RepositoryEntry, RepositoryEntryKind, RepositoryInventory, RustSourceFile},
};

const MAX_CARGO_MANIFESTS: usize = 10_000;
const MAX_RUST_FILES: usize = 20_000;
const MAX_RUST_SOURCE_BYTES: usize = 2 * 1024 * 1024;
const MAX_TOTAL_RUST_SOURCE_BYTES: usize = 128 * 1024 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RepositoryInventoryError(pub(super) String);

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
    let (root, entries) = scan_repository(root, &contract.repository.exclude, false)?;
    let mut rust_files = Vec::new();
    let manifests = cargo_manifests(&entries, Some(contract), &contract.repository.exclude)?;
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

#[cfg(test)]
pub(crate) fn inventory_cargo_repository(
    root: &Path,
) -> Result<RepositoryInventory, RepositoryInventoryError> {
    inventory_selected_cargo_repository(root, &[])
}

pub(crate) fn inventory_selected_cargo_repository(
    root: &Path,
    exclusions: &[String],
) -> Result<RepositoryInventory, RepositoryInventoryError> {
    let (root, entries) = scan_repository(root, exclusions, false)?;
    let manifest_paths = cargo_manifests(&entries, None, exclusions)?;
    Ok(RepositoryInventory {
        root,
        entries,
        rust_files: Vec::new(),
        manifest_paths,
    })
}

fn cargo_manifests(
    entries: &[RepositoryEntry],
    contract: Option<&Contract>,
    exclusions: &[String],
) -> Result<Vec<PathBuf>, RepositoryInventoryError> {
    let mut manifests = Vec::new();
    for entry in entries {
        let excluded = excluded_by(exclusions, &entry.relative);
        if entry.kind != RepositoryEntryKind::File
            || excluded
            || contract.is_some_and(|contract| !under_roots(contract, &entry.relative))
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
