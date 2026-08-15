//! Immutable Git snapshots provide the before-state authority for lock updates.

use zrail_core::{
    ContractBundle, DiffReport, LockFile, compare_architecture_checked, load_contract,
    path::repository_file,
};

use crate::app::{args::UpdateOptions, error::CliError};

use super::git_base::GitSnapshot;

pub(super) fn compare(
    options: &UpdateOptions,
    after: &ContractBundle,
    candidate: &LockFile,
) -> Result<DiffReport, CliError> {
    let common = &options.common;
    let snapshot = GitSnapshot::create(&common.root, &options.base, &common.config, &common.lock)?;
    let before = load_contract(snapshot.root(), &common.config)
        .map_err(|error| CliError::new(format!("load base contract: {error}")))?;
    let lock_path = repository_file(snapshot.root(), &common.lock).map_err(CliError::new)?;
    let before_lock = LockFile::read_optional(&lock_path)
        .map_err(|error| CliError::new(format!("load base lock: {error}")))?;
    Ok(compare_architecture_checked(
        &before.contract,
        &before.sha256,
        before_lock.as_ref(),
        &after.contract,
        &after.sha256,
        Some(candidate),
    ))
}

#[cfg(test)]
#[path = "update_authority_test.rs"]
mod update_authority_test;
