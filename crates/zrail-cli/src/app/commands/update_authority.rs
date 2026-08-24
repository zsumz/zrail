//! Immutable Git snapshots provide the before-state authority for lock updates.

use zrail_core::{
    ContractBundle, DiffReport, LOCK_SEMANTICS, LockFile, compare_architecture_checked,
    compare_lock_epochs, load_contract, repository_file,
};

use crate::app::{args::UpdateOptions, error::CliError};

use super::git_base::GitSnapshot;

pub(super) fn compare(
    options: &UpdateOptions,
    after: &ContractBundle,
    candidate: &LockFile,
) -> Result<DiffReport, CliError> {
    let common = &options.common;
    compare_from_repository(
        &common.root,
        &options.base,
        &common.config,
        &common.lock,
        after,
        candidate,
        options.accept_migration.as_deref(),
    )
}

pub(super) fn compare_current(
    before: &ContractBundle,
    before_lock: Option<&LockFile>,
    after: &ContractBundle,
    candidate: &LockFile,
) -> DiffReport {
    compare_architecture_checked(
        &before.contract,
        &before.sha256,
        before_lock,
        &after.contract,
        &after.sha256,
        Some(candidate),
    )
}

pub(super) fn compare_from_repository(
    authority_root: &std::path::Path,
    base: &std::ffi::OsStr,
    config: &std::path::Path,
    lock: &std::path::Path,
    after: &ContractBundle,
    candidate: &LockFile,
    accept_migration: Option<&str>,
) -> Result<DiffReport, CliError> {
    let snapshot = GitSnapshot::create(authority_root, base, config, lock)?;
    let before = load_contract(snapshot.root(), config)
        .map_err(|error| CliError::new(format!("load base contract: {error}")))?;
    let lock_path = repository_file(snapshot.root(), lock).map_err(CliError::new)?;
    let before_lock = LockFile::read_optional(&lock_path)
        .map_err(|error| CliError::new(format!("load base lock: {error}")))?;
    let migrated = match before_lock.as_ref() {
        Some(old) if old.semantics != LOCK_SEMANTICS => {
            let repository = GitSnapshot::create_repository(
                authority_root,
                std::ffi::OsStr::new(snapshot.commit()),
            )?;
            let reanalyzed = zrail_rust::build_lock(repository.root(), config)
                .map_err(|error| CliError::new(format!("reanalyze migration base: {error}")))?;
            let report = compare_lock_epochs(old, &reanalyzed)
                .map_err(|error| CliError::new(error.to_string()))?;
            let expected = format!("sha256:{}", report.sha256());
            if accept_migration != Some(expected.as_str()) {
                return Err(CliError::new(format!(
                    "lock semantics migration requires --accept-migration {expected} produced by `zrail migrate-lock --base {}`",
                    base.to_string_lossy()
                )));
            }
            Some(reanalyzed)
        }
        _ => None,
    };
    let before_lock = migrated.as_ref().or(before_lock.as_ref());
    Ok(compare_architecture_checked(
        &before.contract,
        &before.sha256,
        before_lock,
        &after.contract,
        &after.sha256,
        Some(candidate),
    ))
}

#[cfg(test)]
#[path = "update_authority_test.rs"]
mod update_authority_test;
