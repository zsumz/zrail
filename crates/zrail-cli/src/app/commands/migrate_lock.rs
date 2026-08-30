//! Reanalyze one immutable base revision before accepting a semantics epoch.

use zrail_core::{LockFile, compare_lock_epochs, replace_text, repository_file};

use crate::app::{args::MigrateLockOptions, error::CliError};

use super::{
    CommandResult, git_base::GitSnapshot, git_migration, migrate_lock_artifact, migration_bridge,
};

pub(crate) fn migrate_lock(options: &MigrateLockOptions) -> Result<CommandResult, CliError> {
    if let Some(target) = options.target.as_deref() {
        return migrate_across_revisions(options, target);
    }
    migrate_same_revision(options)
}

fn migrate_same_revision(options: &MigrateLockOptions) -> Result<CommandResult, CliError> {
    let snapshot = GitSnapshot::create_repository(&options.root, &options.base)?;
    let contract = zrail_core::load_contract(snapshot.root(), &options.config)
        .map_err(|error| CliError::new(format!("load migration base contract: {error}")))?;
    git_migration::require_submodule_policy(&snapshot, contract.contract.repository.submodules)?;
    let old_path = repository_file(snapshot.root(), &options.lock).map_err(CliError::new)?;
    let old = LockFile::read(&old_path)
        .map_err(|error| CliError::new(format!("load migration base lock: {error}")))?;
    if old.contract_sha256 != contract.sha256 {
        return Err(CliError::new(
            "migration base lock was produced from different contract bytes",
        ));
    }
    let reanalyzed = zrail_rust::build_lock(snapshot.root(), &options.config)
        .map_err(|error| CliError::new(format!("reanalyze migration base: {error}")))?;
    let report =
        compare_lock_epochs(&old, &reanalyzed).map_err(|error| CliError::new(error.to_string()))?;
    let digest = report.sha256();
    let rendered =
        migrate_lock_artifact::render(snapshot.commit(), &contract.sha256, &digest, &report)?;
    write_report(options, &rendered, &digest, &snapshot, false)
}

fn migrate_across_revisions(
    options: &MigrateLockOptions,
    target: &std::ffi::OsStr,
) -> Result<CommandResult, CliError> {
    let bridge = migration_bridge::build(
        &options.root,
        &options.base,
        target,
        &options.config,
        &options.lock,
    )?;
    let digest = bridge.report.sha256();
    let rendered = bridge
        .report
        .json()
        .map_err(|error| CliError::new(format!("serialize lock migration bridge: {error}")))?;
    write_report(options, &rendered, &digest, &bridge.target_snapshot, true)
}

fn write_report(
    options: &MigrateLockOptions,
    rendered: &str,
    digest: &str,
    target: &GitSnapshot,
    bridged: bool,
) -> Result<CommandResult, CliError> {
    let output = repository_file(&options.root, &options.output).map_err(CliError::new)?;
    let config = repository_file(&options.root, &options.config).map_err(CliError::new)?;
    let lock = repository_file(&options.root, &options.lock).map_err(CliError::new)?;
    if output == config || output == lock {
        return Err(CliError::new(
            "migration report output may not replace zrail.toml or zrail.lock",
        ));
    }
    git_migration::require_report_output(target, &options.output)?;
    replace_text(&output, rendered)
        .map_err(|error| CliError::new(format!("write {}: {error}", output.display())))?;
    let report = if bridged {
        format!(" --migration-report {}", options.output.display())
    } else {
        String::new()
    };
    Ok(CommandResult::success(format!(
        "Wrote {}\naccept with: --accept-migration sha256:{}{}\n",
        output.display(),
        digest,
        report
    )))
}

#[cfg(test)]
#[path = "migrate_lock_test.rs"]
mod migrate_lock_test;
