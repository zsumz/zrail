//! Git-backed recovery when a lock no longer matches current contract bytes.

use std::{ffi::OsString, path::Path};

use zrail_core::{LockFile, normalize_relative, repository_file};

use crate::app::{args::MigrateLockOptions, error::CliError};

use super::{CommandResult, git_base::GitSnapshot, git_process};

mod render;

const MAX_HISTORY_REVISIONS: usize = 2_048;
const MAX_HISTORY_BYTES: usize = 256 * 1024;

pub(super) fn discover(options: &MigrateLockOptions) -> Result<CommandResult, CliError> {
    let lock_path = repository_file(&options.root, &options.lock).map_err(CliError::new)?;
    let lock = LockFile::read(&lock_path)
        .map_err(|error| CliError::new(format!("load current lock: {error}")))?;
    let context = inspect(
        &options.root,
        &options.config,
        &options.lock,
        &lock.contract_sha256,
    )?;
    let exit_code = i32::from(context.candidates.is_empty());
    Ok(CommandResult::status(
        render::discovery(&context, &lock.contract_sha256),
        exit_code,
    ))
}

pub(super) fn mismatch(
    options: &MigrateLockOptions,
    lock_digest: &str,
    selected_digest: &str,
) -> CliError {
    mismatch_at(
        &options.root,
        &options.config,
        &options.lock,
        lock_digest,
        selected_digest,
    )
}

pub(super) fn mismatch_at(
    root: &Path,
    config: &Path,
    lock: &Path,
    lock_digest: &str,
    selected_digest: &str,
) -> CliError {
    match inspect(root, config, lock, lock_digest) {
        Ok(context) => CliError::new(render::mismatch(&context, lock_digest, selected_digest)),
        Err(error) => CliError::new(format!(
            "migration base lock was produced from different contract bytes\n\
             Lock contract digest: {lock_digest}\n\
             Selected base contract digest: {selected_digest}\n\
             Recovery inspection failed: {error}\n\
             Run `zrail migrate-lock --discover-base` to retry base discovery."
        )),
    }
}

pub(super) struct RecoveryContext {
    pub(super) current_digest: String,
    pub(super) head_digest: Result<String, String>,
    pub(super) candidates: Vec<String>,
}

fn inspect(
    root: &Path,
    config: &Path,
    lock: &Path,
    lock_digest: &str,
) -> Result<RecoveryContext, CliError> {
    let current = zrail_core::load_contract(root, config)
        .map_err(|error| CliError::new(format!("load current contract: {error}")))?;
    let head_digest = contract_at(root, "HEAD", config, lock)
        .map(|bundle| bundle.sha256)
        .map_err(|error| error.message);
    let candidates = matching_revisions(root, config, lock, lock_digest)?;
    Ok(RecoveryContext {
        current_digest: current.sha256,
        head_digest,
        candidates,
    })
}

fn matching_revisions(
    root: &Path,
    config: &Path,
    lock: &Path,
    expected: &str,
) -> Result<Vec<String>, CliError> {
    let lock_path = normalize_relative(lock).map_err(CliError::new)?;
    let maximum = MAX_HISTORY_REVISIONS + 1;
    let output = git_process::output(
        root,
        &[
            OsString::from("rev-list"),
            OsString::from("--all"),
            OsString::from("--date-order"),
            OsString::from(format!("--max-count={maximum}")),
            OsString::from("--"),
            OsString::from(lock_path),
        ],
        MAX_HISTORY_BYTES,
        "migration base discovery",
    )?;
    let revisions = parse_revisions(&output)?;
    if revisions.len() > MAX_HISTORY_REVISIONS {
        return Err(CliError::new(format!(
            "migration base discovery exceeds the {MAX_HISTORY_REVISIONS}-revision safety limit"
        )));
    }
    let mut matches = Vec::new();
    for revision in revisions {
        if revision_matches(root, &revision, config, lock, expected)? {
            matches.push(revision);
        }
    }
    Ok(matches)
}

fn parse_revisions(output: &[u8]) -> Result<Vec<String>, CliError> {
    let output = std::str::from_utf8(output)
        .map_err(|_| CliError::new("Git returned non-UTF-8 revision history"))?;
    output
        .lines()
        .map(|revision| {
            if matches!(revision.len(), 40 | 64)
                && revision.bytes().all(|byte| byte.is_ascii_hexdigit())
            {
                Ok(revision.to_owned())
            } else {
                Err(CliError::new(
                    "Git returned an invalid migration history revision",
                ))
            }
        })
        .collect()
}

fn revision_matches(
    root: &Path,
    revision: &str,
    config: &Path,
    lock: &Path,
    expected: &str,
) -> Result<bool, CliError> {
    let snapshot = match GitSnapshot::create(root, revision.as_ref(), config, lock) {
        Ok(snapshot) => snapshot,
        Err(error) if error.message.contains("does not contain") => return Ok(false),
        Err(error) => return Err(error),
    };
    let Ok(contract) = zrail_core::load_contract(snapshot.root(), config) else {
        return Ok(false);
    };
    let Ok(lock_path) = repository_file(snapshot.root(), lock) else {
        return Ok(false);
    };
    let Ok(historical_lock) = LockFile::read(&lock_path) else {
        return Ok(false);
    };
    Ok(contract.sha256 == expected && historical_lock.contract_sha256 == expected)
}

fn contract_at(
    root: &Path,
    revision: &str,
    config: &Path,
    lock: &Path,
) -> Result<zrail_core::ContractBundle, CliError> {
    let snapshot = GitSnapshot::create(root, revision.as_ref(), config, lock)?;
    zrail_core::load_contract(snapshot.root(), config)
        .map_err(|error| CliError::new(format!("load {revision} contract: {error}")))
}

#[cfg(test)]
#[path = "migrate_lock_recovery_test.rs"]
mod migrate_lock_recovery_test;
