//! Render a complete governed-surface audit without writing repository state.

use zrail_rust::governed_surface_report;

use crate::app::{args::CoverageOptions, error::CliError, output::OutputFormat};

use super::CommandResult;

pub(crate) fn coverage(options: &CoverageOptions) -> Result<CommandResult, CliError> {
    let report = governed_surface_report(&options.root, &options.config)
        .map_err(|error| CliError::new(error.to_string()))?;
    let output = match options.format {
        OutputFormat::Human => report.human(),
        OutputFormat::Json => report
            .json()
            .map_err(|error| CliError::new(format!("serialize coverage report: {error}")))?,
    };
    Ok(CommandResult::success(output))
}
