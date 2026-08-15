//! Wildcard contract imports share one bounded, symlink-safe TOML inventory.

use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::input::{MAX_DIRECTORY_DEPTH, MAX_REPOSITORY_ENTRIES};

use super::ContractError;

pub(super) fn has_wildcard(value: &str) -> bool {
    value.bytes().any(|byte| matches!(byte, b'*' | b'?'))
}

pub(super) fn toml_files(root: &Path) -> Result<Vec<PathBuf>, ContractError> {
    let mut files = Vec::new();
    let mut inspected = 0;
    collect(root, root, &mut files, 0, &mut inspected)?;
    files.sort();
    Ok(files)
}

fn collect(
    root: &Path,
    current: &Path,
    files: &mut Vec<PathBuf>,
    depth: usize,
    inspected: &mut usize,
) -> Result<(), ContractError> {
    if depth > MAX_DIRECTORY_DEPTH {
        return Err(ContractError::one(format!(
            "contract discovery exceeds the {MAX_DIRECTORY_DEPTH}-directory-depth safety limit"
        )));
    }
    let entries = fs::read_dir(current)
        .map_err(|error| ContractError::one(format!("read {}: {error}", current.display())))?;
    for entry in entries {
        if *inspected == MAX_REPOSITORY_ENTRIES {
            return Err(ContractError::one(format!(
                "contract discovery exceeds the {MAX_REPOSITORY_ENTRIES}-entry safety limit"
            )));
        }
        *inspected += 1;
        let path = entry
            .map_err(|error| ContractError::one(format!("read entry: {error}")))?
            .path();
        let metadata = fs::symlink_metadata(&path)
            .map_err(|error| ContractError::one(format!("inspect {}: {error}", path.display())))?;
        if metadata.file_type().is_symlink() {
            continue;
        }
        if metadata.is_dir() {
            let name = path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("");
            if !matches!(name, ".git" | ".zrail" | "target") {
                collect(root, &path, files, depth + 1, inspected)?;
            }
        } else if metadata.is_file()
            && path
                .extension()
                .is_some_and(|extension| extension == "toml")
            && path.starts_with(root)
        {
            files.push(path);
        }
    }
    Ok(())
}
