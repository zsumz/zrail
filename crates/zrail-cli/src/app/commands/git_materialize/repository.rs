//! Full-repository snapshots retain every bounded regular Git blob and file mode.

use std::{collections::BTreeMap, ffi::OsString, path::Path};

use crate::app::error::CliError;

use super::{
    TemporaryRoot,
    filesystem::{set_executable, write_new},
};
use crate::app::commands::{git_base::TreeEntry, git_process};

const MAX_GIT_BLOB_BYTES: usize = 64 * 1024 * 1024;
const MAX_GIT_SNAPSHOT_BYTES: usize = 1024 * 1024 * 1024;

pub(super) fn materialize(
    repository: &Path,
    tree: &BTreeMap<String, TreeEntry>,
) -> Result<TemporaryRoot, CliError> {
    let temporary = TemporaryRoot::create()?;
    let mut total = 0_usize;
    for (path, entry) in tree {
        validate_entry(path, entry, &mut total)?;
        let bytes = git_process::output(
            repository,
            &[
                OsString::from("cat-file"),
                OsString::from("blob"),
                OsString::from(&entry.object),
            ],
            MAX_GIT_BLOB_BYTES,
            "base repository object read",
        )?;
        if bytes.len() != entry.size {
            return Err(CliError::new(format!(
                "Git base object size changed while reading {path}"
            )));
        }
        let destination = temporary.path().join(path);
        write_new(&destination, &bytes)?;
        set_executable(&destination, entry.mode == "100755")?;
    }
    Ok(temporary)
}

fn validate_entry(path: &str, entry: &TreeEntry, total: &mut usize) -> Result<(), CliError> {
    if !entry.is_regular() {
        return Err(CliError::new(format!(
            "full Git base migration cannot silently omit non-regular entry: {path}"
        )));
    }
    if entry.size > MAX_GIT_BLOB_BYTES {
        return Err(CliError::new(format!(
            "Git base blob exceeds the {MAX_GIT_BLOB_BYTES}-byte migration limit: {path}"
        )));
    }
    *total = total
        .checked_add(entry.size)
        .ok_or_else(|| CliError::new("Git base snapshot byte count overflowed"))?;
    if *total > MAX_GIT_SNAPSHOT_BYTES {
        return Err(CliError::new(format!(
            "Git base snapshot exceeds the {MAX_GIT_SNAPSHOT_BYTES}-byte migration limit"
        )));
    }
    Ok(())
}
