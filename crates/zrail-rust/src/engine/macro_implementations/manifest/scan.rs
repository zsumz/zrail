//! Targeted input traversal ignores source exclusions but never scans unrelated package trees.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use zrail_core::{
    MAX_DIRECTORY_DEPTH, MAX_REPOSITORY_ENTRIES, glob_matches, normalize_relative,
    repository_relative,
};

use crate::{
    cargo::{CargoWorkspace, Package},
    inventory::{RepositoryEntry, RepositoryEntryKind},
};

use super::{CheckError, MAX_IMPLEMENTATION_INPUTS, reserved};

pub(super) fn inputs(
    root: &Path,
    cargo: &CargoWorkspace,
    packages: &[&Package],
    patterns: &BTreeSet<String>,
    compile_inputs: &BTreeSet<String>,
) -> Result<Vec<RepositoryEntry>, CheckError> {
    let selected = packages
        .iter()
        .map(|package| package.directory.as_str())
        .collect::<BTreeSet<_>>();
    let excluded_packages = cargo
        .packages
        .iter()
        .filter(|package| !selected.contains(package.directory.as_str()))
        .map(|package| package.directory.clone())
        .collect();
    let mut scan = Scan {
        root,
        excluded_packages,
        entries: BTreeMap::new(),
        visited: BTreeSet::new(),
        inspected: 0,
    };
    for directory in selected {
        scan.start(directory, None)?;
    }
    for path in ["Cargo.toml", "Cargo.lock"]
        .into_iter()
        .chain(compile_inputs.iter().map(String::as_str))
    {
        scan.start(path, Some(path))?;
    }
    for pattern in patterns {
        let prefix = pattern
            .split('/')
            .take_while(|part| !part.contains(['*', '?']))
            .collect::<Vec<_>>()
            .join("/");
        scan.start(&prefix, Some(pattern))?;
    }
    Ok(scan.entries.into_values().collect())
}

struct Scan<'a> {
    root: &'a Path,
    excluded_packages: BTreeSet<String>,
    entries: BTreeMap<String, RepositoryEntry>,
    visited: BTreeSet<(String, Option<String>)>,
    inspected: usize,
}

impl Scan<'_> {
    fn start(&mut self, path: &str, pattern: Option<&str>) -> Result<(), CheckError> {
        let path = normalize_relative(Path::new(path)).map_err(CheckError::from_message)?;
        if reserved(&path) {
            return Ok(());
        }
        // Fixed prefixes and literal includes must not enter through a symlink,
        // including one that points back inside the repository.
        let parts = path
            .split('/')
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>();
        let mut parent = self.root.to_path_buf();
        for part in parts.iter().take(parts.len().saturating_sub(1)) {
            parent.push(part);
            let metadata = match fs::symlink_metadata(&parent) {
                Ok(metadata) => metadata,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
                Err(error) => return Err(failure(&parent, &error)),
            };
            if !metadata.is_dir() || metadata.file_type().is_symlink() {
                return Err(CheckError::from_message(format!(
                    "repository macro input prefix is not an ordinary directory (symlinks are unsupported): {}",
                    parent.display()
                )));
            }
        }
        self.visit(&path, pattern)
    }

    fn visit(&mut self, relative: &str, pattern: Option<&str>) -> Result<(), CheckError> {
        if reserved(relative) || (pattern.is_none() && self.excluded_packages.contains(relative)) {
            return Ok(());
        }
        if !self
            .visited
            .insert((relative.into(), pattern.map(str::to_owned)))
        {
            return Ok(());
        }
        let depth = relative.split('/').filter(|part| !part.is_empty()).count();
        if depth > MAX_DIRECTORY_DEPTH {
            return Err(CheckError::from_message(format!(
                "repository macro inputs exceed the {MAX_DIRECTORY_DEPTH}-directory-depth safety limit"
            )));
        }
        let path = self.root.join(relative);
        let metadata = match fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(failure(&path, &error)),
        };
        if metadata.file_type().is_symlink() {
            return Err(CheckError::from_message(format!(
                "repository macro input {relative:?} is not a regular file (symlink)"
            )));
        }
        if metadata.is_dir() {
            self.directory(&path, pattern)
        } else if metadata.is_file() {
            if pattern.is_none_or(|pattern| glob_matches(pattern, relative)) {
                if !self.entries.contains_key(relative)
                    && self.entries.len() == MAX_IMPLEMENTATION_INPUTS
                {
                    return Err(CheckError::from_message(format!(
                        "macro implementation exceeds the {MAX_IMPLEMENTATION_INPUTS}-input safety limit"
                    )));
                }
                self.entries.insert(
                    relative.into(),
                    RepositoryEntry {
                        relative: relative.into(),
                        absolute: path,
                        kind: RepositoryEntryKind::File,
                    },
                );
            }
            Ok(())
        } else {
            Err(CheckError::from_message(format!(
                "repository macro input {relative:?} is not a regular file"
            )))
        }
    }

    fn directory(&mut self, path: &Path, pattern: Option<&str>) -> Result<(), CheckError> {
        let mut children = Vec::new();
        for child in fs::read_dir(path).map_err(|error| failure(path, &error))? {
            if self.inspected == MAX_REPOSITORY_ENTRIES {
                return Err(CheckError::from_message(format!(
                    "repository macro inputs exceed the {MAX_REPOSITORY_ENTRIES}-entry safety limit"
                )));
            }
            self.inspected += 1;
            let child = child.map_err(|error| failure(path, &error))?;
            children.push(
                repository_relative(self.root, &child.path()).map_err(CheckError::from_message)?,
            );
        }
        children.sort();
        for child in children {
            self.visit(&child, pattern)?;
        }
        Ok(())
    }
}

fn failure(path: &Path, error: &std::io::Error) -> CheckError {
    CheckError::from_message(format!(
        "inspect repository macro input {}: {error}",
        path.display()
    ))
}
