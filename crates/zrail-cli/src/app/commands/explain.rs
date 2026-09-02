//! Report the rails that apply to one repository path.

use std::path::Path;

use zrail_rust::{explain_hypothetical_path, explain_path};

use crate::app::{args::CommonOptions, error::CliError, output::OutputFormat};

use super::CommandResult;

pub(crate) fn explain(
    options: &CommonOptions,
    path: &Path,
    hypothetical: bool,
) -> Result<CommandResult, CliError> {
    let report = if hypothetical {
        explain_hypothetical_path(&options.root, &options.config, path)
    } else {
        explain_path(&options.root, &options.config, path)
    }
    .map_err(|error| CliError::new(error.to_string()))?;
    let text = match options.format {
        OutputFormat::Human => report.human(),
        OutputFormat::Json => report
            .json()
            .map_err(|error| CliError::new(format!("serialize path explanation: {error}")))?,
    };
    Ok(CommandResult::success(text))
}
