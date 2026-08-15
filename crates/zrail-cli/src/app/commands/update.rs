//! Resolve exact repository state without allowing the lock to waive hard rails.

use std::path::Path;

use zrail_core::{
    DiffReport, LockFile, ReportStatus, compare_architecture_checked, load_contract,
    path::repository_file,
};
use zrail_rust::{build_lock, check_repository_with_lock};

use crate::app::{
    args::UpdateOptions,
    error::CliError,
    output::{OutputFormat, json_escape},
};

use super::CommandResult;

pub(crate) fn update(options: &UpdateOptions) -> Result<CommandResult, CliError> {
    let common = &options.common;
    let candidate = build_lock(&common.root, &common.config)
        .map_err(|error| CliError::new(error.to_string()))?;
    let checked = check_repository_with_lock(&common.root, &common.config, &candidate)
        .map_err(|error| CliError::new(error.to_string()))?;
    if checked.report.status != ReportStatus::Pass {
        let text = match common.format {
            OutputFormat::Human => checked.report.human(),
            OutputFormat::Json => checked
                .report
                .json()
                .map_err(|error| CliError::new(format!("serialize report: {error}")))?,
        };
        return Ok(CommandResult::status(text, 1));
    }
    let destination = repository_file(&common.root, &common.lock).map_err(CliError::new)?;
    let current = match read_current_lock(&destination) {
        Ok(lock) => lock,
        Err(_) if options.accept_grants => None,
        Err(error) => {
            return Ok(CommandResult::status(
                unreadable_lock(&error, common.format),
                1,
            ));
        }
    };
    let bundle = load_contract(&common.root, &common.config)
        .map_err(|error| CliError::new(error.to_string()))?;
    let changes = compare_architecture_checked(
        &bundle.contract,
        &bundle.sha256,
        current.as_ref(),
        &bundle.contract,
        &bundle.sha256,
        Some(&candidate),
    );
    if changes.denies_grants() && !options.accept_grants {
        return Ok(CommandResult::status(
            refused_changes(&changes, common.format)?,
            1,
        ));
    }
    candidate
        .write(&destination)
        .map_err(|error| CliError::new(error.to_string()))?;
    let text = match common.format {
        OutputFormat::Human => format!(
            "Updated {}\npackages: {}\ngenerated: {}\ngates: {}\nratchets: {}\n",
            destination.display(),
            candidate.packages.len(),
            candidate.generated.len(),
            candidate.gates.len(),
            candidate.ratchets.len()
        ),
        OutputFormat::Json => format!(
            concat!(
                "{{\n",
                "  \"schema\": 1,\n",
                "  \"status\": \"updated\",\n",
                "  \"path\": \"{}\",\n",
                "  \"packages\": {},\n",
                "  \"generated\": {},\n",
                "  \"gates\": {},\n",
                "  \"ratchets\": {}\n",
                "}}\n",
            ),
            json_escape(&destination.to_string_lossy()),
            candidate.packages.len(),
            candidate.generated.len(),
            candidate.gates.len(),
            candidate.ratchets.len()
        ),
    };
    Ok(CommandResult::success(text))
}

fn read_current_lock(path: &Path) -> Result<Option<LockFile>, String> {
    LockFile::read_optional(path).map_err(|error| error.to_string())
}

fn refused_changes(report: &DiffReport, format: OutputFormat) -> Result<String, CliError> {
    match format {
        OutputFormat::Human => Ok(format!(
            concat!(
                "zrail update refused gated architecture changes\n\n",
                "{}\n",
                "Rerun with `--accept-grants` to write the candidate lock.\n",
            ),
            report.human()
        )),
        OutputFormat::Json => report
            .json()
            .map_err(|error| CliError::new(format!("serialize architecture diff: {error}"))),
    }
}

fn unreadable_lock(error: &str, format: OutputFormat) -> String {
    match format {
        OutputFormat::Human => format!(
            concat!(
                "zrail update refused to replace unreadable architecture state\n\n",
                "{error}\n",
                "Rerun with `--accept-grants` to replace it explicitly.\n",
            ),
            error = error,
        ),
        OutputFormat::Json => format!(
            concat!(
                "{{\n",
                "  \"schema\": 1,\n",
                "  \"status\": \"refused\",\n",
                "  \"error\": \"{}\"\n",
                "}}\n",
            ),
            json_escape(error)
        ),
    }
}

#[cfg(test)]
#[path = "update_test.rs"]
mod update_test;
