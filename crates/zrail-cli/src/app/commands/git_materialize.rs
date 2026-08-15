//! Minimal bounded filesystem snapshots for Git-backed architecture inputs.

use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    ffi::OsString,
    fs::{self, OpenOptions},
    io::Write as _,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

use zrail_core::{
    MAX_CONTRACT_BYTES, MAX_CONTRACT_FILES, MAX_IMPORT_DIRECTIVES, contract_imports,
    input::MAX_INPUT_BYTES,
    path::{glob_matches, normalize_relative},
};

use crate::app::error::CliError;

use super::{git_base::TreeEntry, git_process};

static TEMPORARY_SEQUENCE: AtomicUsize = AtomicUsize::new(0);

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

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), CliError> {
    let parent = path
        .parent()
        .ok_or_else(|| CliError::new(format!("snapshot path has no parent: {}", path.display())))?;
    fs::create_dir_all(parent)
        .map_err(|error| CliError::new(format!("create {}: {error}", parent.display())))?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| CliError::new(format!("create {}: {error}", path.display())))?;
    file.write_all(bytes)
        .map_err(|error| CliError::new(format!("write {}: {error}", path.display())))?;
    file.sync_all()
        .map_err(|error| CliError::new(format!("sync {}: {error}", path.display())))
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

#[derive(Debug)]
pub(super) struct TemporaryRoot(PathBuf);

impl TemporaryRoot {
    fn create() -> Result<Self, CliError> {
        let base = std::env::temp_dir();
        for _ in 0..100 {
            let sequence = TEMPORARY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let path = base.join(format!("zrail-git-{}-{sequence}", std::process::id()));
            match create_private_directory(&path) {
                Ok(()) => return Ok(Self(path)),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(error) => {
                    return Err(CliError::new(format!("create {}: {error}", path.display())));
                }
            }
        }
        Err(CliError::new("create Git snapshot: name collision"))
    }

    pub(super) fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TemporaryRoot {
    fn drop(&mut self) {
        let _removed = fs::remove_dir_all(&self.0);
    }
}

fn create_private_directory(path: &Path) -> std::io::Result<()> {
    let mut builder = fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt as _;
        builder.mode(0o700);
    }
    builder.create(path)
}
