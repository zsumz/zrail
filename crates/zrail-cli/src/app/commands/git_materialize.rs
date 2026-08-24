//! Minimal bounded filesystem snapshots for Git-backed architecture inputs.

use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    ffi::OsString,
    path::Path,
};

use zrail_core::{
    MAX_CONTRACT_BYTES, MAX_CONTRACT_FILES, MAX_IMPORT_DIRECTIVES, MAX_INPUT_BYTES,
    contract_imports, glob_matches, normalize_relative,
};

use crate::app::error::CliError;

use super::{git_base::TreeEntry, git_process};

#[path = "git_materialize/filesystem.rs"]
mod filesystem;
#[path = "git_materialize/repository.rs"]
mod repository;

pub(super) use filesystem::TemporaryRoot;
use filesystem::write_new;

pub(super) fn materialize(
    repository: &Path,
    tree: &BTreeMap<String, TreeEntry>,
    config: &Path,
    lock: &Path,
) -> Result<TemporaryRoot, CliError> {
    let temporary = TemporaryRoot::create()?;
    materialize_contracts(repository, temporary.path(), tree, config)?;
    materialize_optional(repository, temporary.path(), tree, lock)?;
    Ok(temporary)
}

pub(super) fn materialize_repository(
    repository: &Path,
    tree: &BTreeMap<String, TreeEntry>,
) -> Result<TemporaryRoot, CliError> {
    repository::materialize(repository, tree)
}

fn materialize_contracts(
    repository: &Path,
    destination: &Path,
    tree: &BTreeMap<String, TreeEntry>,
    config: &Path,
) -> Result<(), CliError> {
    let config = normalize_required(config, "configuration")?;
    let mut queue = VecDeque::from([config]);
    let mut seen = BTreeSet::new();
    let mut bytes = 0_usize;
    let mut directives = 0_usize;
    while let Some(path) = queue.pop_front() {
        if !seen.insert(path.clone()) {
            continue;
        }
        if seen.len() > MAX_CONTRACT_FILES {
            return Err(CliError::new(format!(
                "Git base contract exceeds the {MAX_CONTRACT_FILES}-file safety limit"
            )));
        }
        let source = materialize_text(repository, destination, tree, &path)?;
        bytes = bytes
            .checked_add(source.len())
            .ok_or_else(|| CliError::new("Git base contract byte count overflowed"))?;
        if bytes > MAX_CONTRACT_BYTES {
            return Err(CliError::new(format!(
                "Git base contract exceeds the {MAX_CONTRACT_BYTES}-byte safety limit"
            )));
        }
        let imports =
            contract_imports(&source, &path).map_err(|error| CliError::new(error.to_string()))?;
        directives = directives
            .checked_add(imports.len())
            .ok_or_else(|| CliError::new("Git base import count overflowed"))?;
        if directives > MAX_IMPORT_DIRECTIVES {
            return Err(CliError::new(format!(
                "Git base contract exceeds the {MAX_IMPORT_DIRECTIVES}-directive safety limit"
            )));
        }
        enqueue_imports(imports, tree, &mut queue)?;
    }
    Ok(())
}

fn enqueue_imports(
    imports: Vec<String>,
    tree: &BTreeMap<String, TreeEntry>,
    queue: &mut VecDeque<String>,
) -> Result<(), CliError> {
    for import in imports {
        let import = normalize_required(Path::new(&import), "contract import")?;
        if has_wildcard(&import) {
            let matches = tree
                .iter()
                .filter(|(path, entry)| {
                    entry.is_regular()
                        && Path::new(path)
                            .extension()
                            .is_some_and(|value| value == "toml")
                        && glob_matches(&import, path)
                })
                .map(|(path, _)| path.clone())
                .collect::<Vec<_>>();
            if matches.is_empty() {
                return Err(CliError::new(format!(
                    "contract import {import:?} matched no files in the Git base"
                )));
            }
            queue.extend(matches);
        } else {
            queue.push_back(import);
        }
    }
    Ok(())
}

fn materialize_optional(
    repository: &Path,
    destination: &Path,
    tree: &BTreeMap<String, TreeEntry>,
    path: &Path,
) -> Result<(), CliError> {
    let path = normalize_required(path, "lock")?;
    if tree.contains_key(&path) {
        let _source = materialize_text(repository, destination, tree, &path)?;
    }
    Ok(())
}

fn materialize_text(
    repository: &Path,
    destination: &Path,
    tree: &BTreeMap<String, TreeEntry>,
    path: &str,
) -> Result<String, CliError> {
    let entry = tree
        .get(path)
        .ok_or_else(|| CliError::new(format!("Git base does not contain {path}")))?;
    if !entry.is_regular() {
        return Err(CliError::new(format!(
            "Git base architecture input is not a regular file: {path}"
        )));
    }
    if entry.size > MAX_INPUT_BYTES {
        return Err(CliError::new(format!(
            "Git base architecture input exceeds the {MAX_INPUT_BYTES}-byte safety limit: {path}"
        )));
    }
    let bytes = git_process::output(
        repository,
        &[
            OsString::from("cat-file"),
            OsString::from("blob"),
            OsString::from(&entry.object),
        ],
        MAX_INPUT_BYTES,
        "base object read",
    )?;
    if bytes.len() != entry.size {
        return Err(CliError::new(format!(
            "Git base object size changed while reading {path}"
        )));
    }
    let source = String::from_utf8(bytes)
        .map_err(|_| CliError::new(format!("Git base architecture input is not UTF-8: {path}")))?;
    write_new(&destination.join(path), source.as_bytes())?;
    Ok(source)
}

fn normalize_required(path: &Path, label: &str) -> Result<String, CliError> {
    let normalized = normalize_relative(path).map_err(CliError::new)?;
    if normalized.is_empty() {
        Err(CliError::new(format!(
            "Git base {label} path may not be empty"
        )))
    } else {
        Ok(normalized)
    }
}

fn has_wildcard(value: &str) -> bool {
    value.bytes().any(|byte| matches!(byte, b'*' | b'?'))
}
