//! Contained exact-path probes never infer absence through unresolved directory links.

use std::{fs, io::ErrorKind, path::Path};

use zrail_core::{RepositoryEntryMode, repository_relative};

use crate::inventory::{RepositoryEntry, RepositoryEntryKind};

use super::model::GovernedRepositoryFileEntry;

pub(super) fn probe(root: &Path, relative: &str) -> Result<Option<RepositoryEntry>, String> {
    let mut current = root.to_path_buf();
    let mut components = Path::new(relative).components().peekable();
    while let Some(component) = components.next() {
        current.push(component);
        let metadata = match fs::symlink_metadata(&current) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(format!("inspect {relative:?}: {error}")),
        };
        if components.peek().is_none() {
            return Ok(Some(RepositoryEntry {
                relative: relative.into(),
                absolute: current,
                kind: kind(&metadata),
            }));
        }
        if metadata.file_type().is_symlink() {
            current = contained(root, &current)?;
            if !current.is_dir() {
                return Ok(None);
            }
        } else if !metadata.is_dir() {
            return Ok(None);
        }
    }
    Err("repository-file selection requires a nonempty relative path".into())
}

pub(super) fn observe(
    root: &Path,
    entry: &RepositoryEntry,
    mode: RepositoryEntryMode,
) -> Result<Option<GovernedRepositoryFileEntry>, String> {
    let needs_target =
        entry.kind == RepositoryEntryKind::Symlink && mode != RepositoryEntryMode::Any;
    let target = if needs_target {
        contained(root, &entry.absolute)?
    } else {
        entry.absolute.clone()
    };
    let target_kind = if needs_target {
        kind(&fs::metadata(&target).map_err(|error| format!("inspect link target: {error}"))?)
    } else {
        entry.kind
    };
    let selected = match mode {
        RepositoryEntryMode::File => target_kind == RepositoryEntryKind::File,
        RepositoryEntryMode::Directory => target_kind == RepositoryEntryKind::Directory,
        RepositoryEntryMode::Any => true,
    };
    if !selected {
        return Ok(None);
    }
    let target = repository_relative(root, &target)?;
    Ok(Some(GovernedRepositoryFileEntry {
        path: entry.relative.clone(),
        kind: match entry.kind {
            RepositoryEntryKind::File => "file",
            RepositoryEntryKind::Directory => "directory",
            RepositoryEntryKind::Symlink => "symlink",
            RepositoryEntryKind::Other => "other",
        }
        .into(),
        resolved_path: (target != entry.relative).then_some(target),
        sha256: None,
        bytes: None,
        valid_utf8: None,
        satisfied: true,
        literal_count: None,
        literal_offsets: Vec::new(),
        omitted_literal_offsets: 0,
        forbidden_names: Vec::new(),
    }))
}

pub(super) fn contained(root: &Path, path: &Path) -> Result<std::path::PathBuf, String> {
    let target = fs::canonicalize(path)
        .map_err(|error| format!("resolve repository file {}: {error}", path.display()))?;
    if !target.starts_with(root) {
        return Err(format!(
            "repository file escapes through a link: {}",
            path.display()
        ));
    }
    Ok(target)
}

fn kind(metadata: &fs::Metadata) -> RepositoryEntryKind {
    if metadata.file_type().is_symlink() {
        RepositoryEntryKind::Symlink
    } else if metadata.is_dir() {
        RepositoryEntryKind::Directory
    } else if metadata.is_file() {
        RepositoryEntryKind::File
    } else {
        RepositoryEntryKind::Other
    }
}
