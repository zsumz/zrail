//! Reanalyze one immutable base revision before accepting a semantics epoch.

use zrail_core::{LockFile, compare_lock_epochs, replace_text, repository_file};

use crate::app::{args::MigrateLockOptions, error::CliError};

use super::{CommandResult, git_base::GitSnapshot, migrate_lock_artifact};

pub(crate) fn migrate_lock(options: &MigrateLockOptions) -> Result<CommandResult, CliError> {
    let snapshot = GitSnapshot::create_repository(&options.root, &options.base)?;
    let contract = zrail_core::load_contract(snapshot.root(), &options.config)
        .map_err(|error| CliError::new(format!("load migration base contract: {error}")))?;
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
    let output = repository_file(&options.root, &options.output).map_err(CliError::new)?;
    let config = repository_file(&options.root, &options.config).map_err(CliError::new)?;
    let lock = repository_file(&options.root, &options.lock).map_err(CliError::new)?;
    if output == config || output == lock {
        return Err(CliError::new(
            "migration report output may not replace zrail.toml or zrail.lock",
        ));
    }
    replace_text(&output, &rendered)
        .map_err(|error| CliError::new(format!("write {}: {error}", output.display())))?;
    Ok(CommandResult::success(format!(
        "Wrote {}\naccept with: --accept-migration sha256:{}\n",
        output.display(),
        digest
    )))
}

#[cfg(test)]
#[path = "migrate_lock_test.rs"]
mod migrate_lock_test;
