//! Immutable Git snapshots provide the before-state authority for lock updates.

use zrail_core::{
    ContractBundle, DiffReport, LOCK_SEMANTICS, LockFile, compare_architecture_checked,
    compare_lock_epochs, load_contract, repository_file,
};

use crate::app::{args::UpdateOptions, error::CliError};

use super::{git_base::GitSnapshot, git_migration, migration_bridge};

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
        options.migration_report.as_deref(),
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
    migration_report: Option<&std::path::Path>,
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
            match zrail_rust::build_lock(repository.root(), config) {
                Ok(reanalyzed) => {
                    git_migration::require_submodule_policy(
                        &repository,
                        before.contract.repository.submodules,
                    )?;
                    if migration_report.is_some() {
                        return Err(CliError::new(
                            "--migration-report is only valid for a cross-revision migration",
                        ));
                    }
                    let report = compare_lock_epochs(old, &reanalyzed)
                        .map_err(|error| CliError::new(error.to_string()))?;
                    require_acceptance(
                        accept_migration,
                        &report.sha256(),
                        &format!("--base {}", base.to_string_lossy()),
                    )?;
                    Some(reanalyzed)
                }
                Err(_) => Some(migrate_across_revisions(&CrossRevision {
                    authority_root,
                    base,
                    config,
                    lock,
                    before: &before,
                    after,
                    candidate,
                    acceptance: accept_migration,
                    report_path: migration_report,
                })?),
            }
        }
        _ if migration_report.is_some() => {
            return Err(CliError::new(
                "--migration-report is only valid for a cross-revision migration",
            ));
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

struct CrossRevision<'a> {
    authority_root: &'a std::path::Path,
    base: &'a std::ffi::OsStr,
    config: &'a std::path::Path,
    lock: &'a std::path::Path,
    before: &'a ContractBundle,
    after: &'a ContractBundle,
    candidate: &'a LockFile,
    acceptance: Option<&'a str>,
    report_path: Option<&'a std::path::Path>,
}

fn migrate_across_revisions(inputs: &CrossRevision<'_>) -> Result<LockFile, CliError> {
    let bridge = migration_bridge::build(
        inputs.authority_root,
        inputs.base,
        std::ffi::OsStr::new("HEAD"),
        inputs.config,
        inputs.lock,
    )?;
    if bridge.base_contract.sha256 != inputs.before.sha256 {
        return Err(CliError::new(
            "migration bridge base contract changed while recomputing authority",
        ));
    }
    let report_path = inputs.report_path.ok_or_else(|| {
        CliError::new("cross-revision migration requires --migration-report PATH")
    })?;
    let report = bridge
        .report
        .json()
        .map_err(|error| CliError::new(format!("serialize lock migration bridge: {error}")))?;
    git_migration::require_worktree_target(
        inputs.authority_root,
        &bridge.target_snapshot,
        report_path,
        &report,
    )?;
    if bridge.report.target.contract_sha256 != inputs.after.sha256
        || !bridge.target_lock.same_resolved_state(inputs.candidate)
    {
        return Err(CliError::new(
            "current worktree does not match the reviewed migration target commit",
        ));
    }
    require_acceptance(
        inputs.acceptance,
        &bridge.report.sha256(),
        &format!("--base {} --target HEAD", inputs.base.to_string_lossy()),
    )?;
    // The reviewed bridge accepts its listed resolved-state changes. Rebinding only the
    // digest lets the normal contract comparison independently retain grant authority.
    let mut migrated = bridge.target_lock;
    migrated.contract_sha256.clone_from(&inputs.before.sha256);
    Ok(migrated)
}

fn require_acceptance(
    accepted: Option<&str>,
    report_sha256: &str,
    command_arguments: &str,
) -> Result<(), CliError> {
    let expected = format!("sha256:{report_sha256}");
    if accepted == Some(expected.as_str()) {
        return Ok(());
    }
    Err(CliError::new(format!(
        "lock semantics migration requires --accept-migration {expected} produced by `zrail migrate-lock {command_arguments}`"
    )))
}

#[cfg(test)]
#[path = "update_authority_test.rs"]
mod update_authority_test;
