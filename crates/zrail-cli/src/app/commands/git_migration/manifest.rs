//! Content-bound migration manifests for immutable Git snapshots.

use std::{collections::BTreeSet, fs, path::Path};

use zrail_core::{LockMigrationFileChange, LockMigrationFileState, normalize_relative, sha256_hex};

use crate::app::error::CliError;

use super::super::git_base::GitSnapshot;

pub(in crate::app::commands) fn require_report_output(
    snapshot: &GitSnapshot,
    output: &Path,
) -> Result<(), CliError> {
    let output = normalize_relative(output).map_err(CliError::new)?;
    if output.is_empty() || snapshot.tree.contains_key(&output) {
        Err(CliError::new(
            "migration report output must not replace a tracked target file",
        ))
    } else {
        Ok(())
    }
}

pub(in crate::app::commands) fn changes(
    base: &GitSnapshot,
    target: &GitSnapshot,
) -> Result<Vec<LockMigrationFileChange>, CliError> {
    let paths = base
        .tree
        .keys()
        .chain(target.tree.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    paths
        .into_iter()
        .filter(|path| base.tree.get(path) != target.tree.get(path))
        .map(|path| {
            Ok(LockMigrationFileChange {
                before: file_state(base, &path)?,
                after: file_state(target, &path)?,
                path,
            })
        })
        .collect()
}

fn file_state(
    snapshot: &GitSnapshot,
    path: &str,
) -> Result<Option<LockMigrationFileState>, CliError> {
    let Some(entry) = snapshot.tree.get(path) else {
        return Ok(None);
    };
    let sha256 = if entry.is_regular() {
        let bytes = fs::read(snapshot.root().join(path))
            .map_err(|error| CliError::new(format!("read migration file {path}: {error}")))?;
        sha256_hex(&bytes)
    } else if entry.is_symlink() {
        sha256_hex(&symlink_bytes(
            &fs::read_link(snapshot.root().join(path)).map_err(|error| {
                CliError::new(format!("read migration symlink {path}: {error}"))
            })?,
        ))
    } else if entry.is_gitlink() {
        sha256_hex(format!("gitlink\0{}", entry.object).as_bytes())
    } else {
        return Err(CliError::new(format!(
            "unsupported migration file mode {} at {path}",
            entry.mode
        )));
    };
    Ok(Some(LockMigrationFileState {
        mode: entry.mode.clone(),
        sha256,
    }))
}

#[cfg(unix)]
fn symlink_bytes(path: &Path) -> Vec<u8> {
    use std::os::unix::ffi::OsStrExt as _;
    path.as_os_str().as_bytes().to_vec()
}

#[cfg(not(unix))]
fn symlink_bytes(path: &Path) -> Vec<u8> {
    path.to_string_lossy().as_bytes().to_vec()
}
