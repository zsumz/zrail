//! Compare effective architecture permission across two repository states.

use std::path::Path;

use zrail_core::{LockFile, compare_architecture_checked, load_contract, path::repository_file};

use crate::app::{
    args::{DiffMode, DiffOptions},
    error::CliError,
    output::OutputFormat,
};

use super::{CommandResult, git_base::GitSnapshot};

pub(crate) fn diff(options: &DiffOptions) -> Result<CommandResult, CliError> {
    match &options.mode {
        DiffMode::Base { root, revision } => {
            let snapshot = GitSnapshot::create(root, revision, &options.config, &options.lock)?;
            compare(snapshot.root(), root, options)
        }
        DiffMode::Explicit { before, after } => compare(before, after, options),
    }
}

fn compare(
    before_root: &Path,
    after_root: &Path,
    options: &DiffOptions,
) -> Result<CommandResult, CliError> {
    let before = load_contract(before_root, &options.config)
        .map_err(|error| CliError::new(format!("load before contract: {error}")))?;
    let after = load_contract(after_root, &options.config)
        .map_err(|error| CliError::new(format!("load after contract: {error}")))?;
    let before_lock = optional_lock(before_root, &options.lock)?;
    let after_lock = optional_lock(after_root, &options.lock)?;
    let report = compare_architecture_checked(
        &before.contract,
        &before.sha256,
        before_lock.as_ref(),
        &after.contract,
        &after.sha256,
        after_lock.as_ref(),
    );
    let text = match options.format {
        OutputFormat::Human => report.human(),
        OutputFormat::Json => report
            .json()
            .map_err(|error| CliError::new(format!("serialize architecture diff: {error}")))?,
    };
    let exit_code = i32::from(options.deny_grants && report.denies_grants());
    Ok(CommandResult::status(text, exit_code))
}

fn optional_lock(root: &Path, lock: &Path) -> Result<Option<LockFile>, CliError> {
    let path = repository_file(root, lock).map_err(CliError::new)?;
    LockFile::read_optional(&path).map_err(|error| CliError::new(error.to_string()))
}

#[cfg(test)]
#[path = "diff_test.rs"]
mod diff_test;
