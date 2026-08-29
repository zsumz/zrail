//! Cross-revision migration is reserved for bases the current engine cannot analyze.

use std::{ffi::OsStr, path::Path};

use zrail_core::{
    ContractBundle, LockFile, LockMigrationBridgeReport, LockMigrationRevision, MAX_INPUT_BYTES,
    ReportStatus, compare_lock_epochs_across_revisions, load_contract, read_bytes_with_limit,
    repository_file, sha256_hex,
};

use crate::app::error::CliError;

use super::{git_base::GitSnapshot, git_migration};

pub(super) struct MigrationBridge {
    pub(super) report: LockMigrationBridgeReport,
    pub(super) base_contract: ContractBundle,
    pub(super) target_lock: LockFile,
    pub(super) target_snapshot: GitSnapshot,
}

pub(super) fn build(
    repository: &Path,
    base: &OsStr,
    target: &OsStr,
    config: &Path,
    lock: &Path,
) -> Result<MigrationBridge, CliError> {
    let base_snapshot = GitSnapshot::create_repository(repository, base)?;
    let target_snapshot = GitSnapshot::create_repository(repository, target)?;
    git_migration::require_ancestor(repository, base_snapshot.commit(), target_snapshot.commit())?;
    if base_snapshot.commit() == target_snapshot.commit() {
        return Err(CliError::new(
            "cross-revision migration requires a target commit after the base",
        ));
    }
    let base_contract = load_contract(base_snapshot.root(), config)
        .map_err(|error| CliError::new(format!("load migration base contract: {error}")))?;
    let old_path = repository_file(base_snapshot.root(), lock).map_err(CliError::new)?;
    let old_bytes = read_bytes_with_limit(&old_path, MAX_INPUT_BYTES)
        .map_err(|error| CliError::new(format!("read migration base lock: {error}")))?;
    let old = LockFile::read(&old_path)
        .map_err(|error| CliError::new(format!("load migration base lock: {error}")))?;
    if old.contract_sha256 != base_contract.sha256 {
        return Err(CliError::new(
            "migration base lock was produced from different contract bytes",
        ));
    }
    let base_error = match zrail_rust::build_lock(base_snapshot.root(), config) {
        Ok(_) => {
            return Err(CliError::new(
                "migration base remains analyzable; use the strict same-revision migration",
            ));
        }
        Err(error) => stable_error(&error.to_string(), base_snapshot.root()),
    };
    let target_contract = load_contract(target_snapshot.root(), config)
        .map_err(|error| CliError::new(format!("load migration target contract: {error}")))?;
    let target_old_path = repository_file(target_snapshot.root(), lock).map_err(CliError::new)?;
    let target_old_bytes =
        read_bytes_with_limit(&target_old_path, MAX_INPUT_BYTES).map_err(|_| {
            CliError::new("migration target must retain the exact base lock until acceptance")
        })?;
    if target_old_bytes != old_bytes {
        return Err(CliError::new(
            "migration target must retain the exact base lock until acceptance",
        ));
    }
    let target_lock = zrail_rust::build_lock(target_snapshot.root(), config)
        .map_err(|error| CliError::new(format!("analyze migration target: {error}")))?;
    let checked =
        zrail_rust::check_repository_with_lock(target_snapshot.root(), config, &target_lock)
            .map_err(|error| CliError::new(format!("check migration target: {error}")))?;
    if checked.report.status != ReportStatus::Pass {
        return Err(CliError::new(
            "migration target must pass the current engine before bridging epochs",
        ));
    }
    let migration = compare_lock_epochs_across_revisions(&old, &target_lock)
        .map_err(|error| CliError::new(error.to_string()))?;
    let report = LockMigrationBridgeReport {
        schema: 1,
        base: revision(base_snapshot.commit(), &base_contract, &old)?,
        target: revision(target_snapshot.commit(), &target_contract, &target_lock)?,
        base_analysis_error: base_error,
        changes: git_migration::changes(&base_snapshot, &target_snapshot)?,
        migration,
    };
    Ok(MigrationBridge {
        report,
        base_contract,
        target_lock,
        target_snapshot,
    })
}

fn revision(
    commit: &str,
    contract: &ContractBundle,
    lock: &LockFile,
) -> Result<LockMigrationRevision, CliError> {
    let rendered = lock
        .render()
        .map_err(|error| CliError::new(format!("render migration lock: {error}")))?;
    Ok(LockMigrationRevision {
        commit: commit.into(),
        contract_sha256: contract.sha256.clone(),
        lock_sha256: sha256_hex(rendered.as_bytes()),
    })
}

fn stable_error(error: &str, root: &Path) -> String {
    error.replace(root.to_string_lossy().as_ref(), "<migration-base>")
}

#[cfg(test)]
#[path = "migration_bridge_test.rs"]
mod migration_bridge_test;
