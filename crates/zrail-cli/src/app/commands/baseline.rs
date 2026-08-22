//! Adopt measurable legacy debt without broadening unrelated architecture policy.

use zrail_core::{LockFile, ReportStatus, repository_file};

use crate::app::{args::BaselineOptions, error::CliError};

use super::{CommandResult, baseline_output, baseline_plan, baseline_write, update_authority};

pub(crate) fn baseline(options: &BaselineOptions) -> Result<CommandResult, CliError> {
    let plan = baseline_plan::prepare(
        &options.common.root,
        &options.common.config,
        options.rule.as_deref(),
    )?;
    if plan.report.status != ReportStatus::Pass {
        return Ok(CommandResult::status(
            baseline_output::render(
                &plan,
                None,
                baseline_output::BaselineStatus::Rejected,
                options.common.format,
            ),
            1,
        ));
    }
    let lock_path = repository_file(&plan.root, &options.common.lock).map_err(CliError::new)?;
    let current = match LockFile::read_optional(&lock_path) {
        Ok(lock) => lock,
        Err(_) if options.accept_grants => None,
        Err(error) => {
            return Err(CliError::new(format!(
                "baseline refused to replace unreadable architecture state: {error}"
            )));
        }
    };
    let authority = update_authority::compare_current(
        &plan.before,
        current.as_ref(),
        &plan.after,
        &plan.candidate_lock,
    );
    if options.dry_run {
        return Ok(CommandResult::success(baseline_output::render(
            &plan,
            Some(&authority),
            baseline_output::BaselineStatus::DryRun,
            options.common.format,
        )));
    }
    if authority.denies_grants() && !options.accept_grants {
        return Ok(CommandResult::status(
            baseline_output::render(
                &plan,
                Some(&authority),
                baseline_output::BaselineStatus::Refused,
                options.common.format,
            ),
            1,
        ));
    }
    baseline_write::write(
        &plan.config_path,
        &lock_path,
        &plan.original_contract,
        &plan.patched_contract,
        &plan.candidate_lock,
    )
    .map_err(CliError::new)?;
    Ok(CommandResult::success(baseline_output::render(
        &plan,
        Some(&authority),
        baseline_output::BaselineStatus::Updated,
        options.common.format,
    )))
}

#[cfg(test)]
#[path = "baseline_test.rs"]
mod baseline_test;
