//! Evaluate the repository against its active architecture contract.

use zrail_core::ReportStatus;
use zrail_rust::check_repository_with_limit;

use crate::app::{args::CommonOptions, error::CliError, output::OutputFormat};

use super::CommandResult;

pub(crate) fn check(options: &CommonOptions) -> Result<CommandResult, CliError> {
    let result =
        check_repository_with_limit(&options.root, &options.config, &options.lock, options.limit)
            .map_err(|error| CliError::new(error.to_string()))?;
    let text = match options.format {
        OutputFormat::Human => result.report.human(),
        OutputFormat::Json => result
            .report
            .json()
            .map_err(|error| CliError::new(format!("serialize report: {error}")))?,
    };
    let exit_code = i32::from(result.report.status != ReportStatus::Pass);
    Ok(CommandResult::status(text, exit_code))
}
