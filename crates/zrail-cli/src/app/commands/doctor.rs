//! Validate the contract, repository inventory, and lock location.

use zrail_rust::doctor_repository;

use crate::app::{args::CommonOptions, error::CliError, output::OutputFormat};

use super::CommandResult;

pub(crate) fn doctor(options: &CommonOptions) -> Result<CommandResult, CliError> {
    let report = doctor_repository(&options.root, &options.config, &options.lock)
        .map_err(|error| CliError::new(error.to_string()))?;
    let text = match options.format {
        OutputFormat::Human => report.human(),
        OutputFormat::Json => report
            .json()
            .map_err(|error| CliError::new(format!("serialize doctor report: {error}")))?,
    };
    let exit_code = i32::from(!report.is_ready());
    Ok(CommandResult::status(text, exit_code))
}
