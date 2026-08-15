//! Wildcard contract imports share one bounded, symlink-safe TOML inventory.

use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::input::{MAX_DIRECTORY_DEPTH, MAX_REPOSITORY_ENTRIES};
use crate::path::{MAX_GLOB_PATTERN_BYTES, MAX_GLOB_PATTERN_SEGMENTS, normalize_relative};

use super::ContractError;

pub(super) fn has_wildcard(value: &str) -> bool {
    value.bytes().any(|byte| matches!(byte, b'*' | b'?'))
}

pub(super) fn normalize_import(pattern: &str) -> Result<String, ContractError> {
    let normalized = normalize_relative(Path::new(pattern)).map_err(ContractError::one)?;
    if normalized.is_empty() {
        return Err(ContractError::one("contract import path may not be empty"));
    }
    if normalized.len() > MAX_GLOB_PATTERN_BYTES
        || normalized.split('/').count() > MAX_GLOB_PATTERN_SEGMENTS
    {
        return Err(ContractError::one(
            "contract import pattern exceeds safety limits",
        ));
    }
    Ok(normalized)
}

pub(super) fn fixed_prefix(pattern: &str) -> String {
    pattern
        .split('/')
        .take_while(|segment| !has_wildcard(segment))
        .collect::<Vec<_>>()
        .join("/")
}

pub(super) fn toml_files(
    root: &Path,
    prefix: &str,
    inspected: &mut usize,
) -> Result<Vec<PathBuf>, ContractError> {
    let depth = prefix.split('/').filter(|part| !part.is_empty()).count();
    if depth > MAX_DIRECTORY_DEPTH {
        return Err(ContractError::one(format!(
            "contract discovery exceeds the {MAX_DIRECTORY_DEPTH}-directory-depth safety limit"
        )));
    }
    let Some(start) = traversal_root(root, prefix)? else {
        return Ok(Vec::new());
    };
    let mut files = Vec::new();
    collect(root, &start, &mut files, depth, inspected)?;
    files.sort();
    Ok(files)
}

fn traversal_root(root: &Path, prefix: &str) -> Result<Option<PathBuf>, ContractError> {
    let mut current = root.to_path_buf();
    for component in prefix.split('/').filter(|part| !part.is_empty()) {
        current.push(component);
        if ignored_root_directory(root, &current) {
            return Ok(None);
        }
        let metadata = match fs::symlink_metadata(&current) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => {
                return Err(ContractError::one(format!(
                    "inspect {}: {error}",
                    current.display()
                )));
            }
        };
        if metadata.file_type().is_symlink() {
            return Err(ContractError::one(format!(
                "contract discovery prefix is a symlink: {}",
                current.display()
            )));
        }
        if !metadata.is_dir() {
            return Ok(None);
        }
    }
    Ok(Some(current))
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
    let directory = fs::read_dir(current)
        .map_err(|error| ContractError::one(format!("read {}: {error}", current.display())))?;
    let mut entries = Vec::new();
    for entry in directory {
        if *inspected == MAX_REPOSITORY_ENTRIES {
            return Err(ContractError::one(format!(
                "contract discovery exceeds the {MAX_REPOSITORY_ENTRIES}-entry safety limit"
            )));
        }
        *inspected += 1;
        entries.push(entry.map_err(|error| ContractError::one(format!("read entry: {error}")))?);
    }
    entries.sort_by_key(fs::DirEntry::file_name);
    for entry in entries {
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path)
            .map_err(|error| ContractError::one(format!("inspect {}: {error}", path.display())))?;
        if metadata.file_type().is_symlink() {
            continue;
        }
        if metadata.is_dir() {
            if !ignored_root_directory(root, &path) {
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

fn ignored_root_directory(root: &Path, path: &Path) -> bool {
    path.parent() == Some(root)
        && path
            .file_name()
            .and_then(|value| value.to_str())
            .is_some_and(|name| matches!(name, ".git" | ".zrail" | "target"))
}

#[cfg(test)]
#[path = "discover_test.rs"]
mod discover_test;
