//! Render exact test-mirror plans and verify current schema-2 receipts.

use zrail_core::{ReportStatus, read_text, repository_file};
use zrail_rust::{render_test_mirror_receipts, test_mirror_plan, verify_test_mirrors};

use crate::app::{
    args::{MirrorsAction, MirrorsOptions},
    error::CliError,
    output::OutputFormat,
};

use super::CommandResult;

pub(crate) fn mirrors(options: &MirrorsOptions) -> Result<CommandResult, CliError> {
    match &options.action {
        MirrorsAction::Plan => render_plan(options),
        MirrorsAction::Verify { plan } => verify_plan(options, plan),
        MirrorsAction::Receipts { plan, results } => render_receipts(options, plan, results),
    }
}

fn render_receipts(
    options: &MirrorsOptions,
    plan_path: &std::path::Path,
    result_path: &std::path::Path,
) -> Result<CommandResult, CliError> {
    let plan_path = repository_file(&options.root, plan_path).map_err(CliError::new)?;
    let result_path = repository_file(&options.root, result_path).map_err(CliError::new)?;
    let plan = read_text(&plan_path)
        .map_err(|error| CliError::new(format!("read mirror plan: {error}")))?;
    let results = read_text(&result_path)
        .map_err(|error| CliError::new(format!("read mirror results: {error}")))?;
    let bundle = render_test_mirror_receipts(&options.root, &options.config, &plan, &results)
        .map_err(|error| CliError::new(error.to_string()))?;
    let output = match options.format {
        OutputFormat::Human => bundle.human(),
        OutputFormat::Json => bundle
            .json()
            .map_err(|error| CliError::new(format!("serialize mirror receipt bundle: {error}")))?,
    };
    Ok(CommandResult::success(output))
}

fn render_plan(options: &MirrorsOptions) -> Result<CommandResult, CliError> {
    let plan = test_mirror_plan(&options.root, &options.config)
        .map_err(|error| CliError::new(error.to_string()))?;
    let output = match options.format {
        OutputFormat::Human => plan.human(),
        OutputFormat::Json => plan
            .json()
            .map_err(|error| CliError::new(format!("serialize mirror plan: {error}")))?,
    };
    Ok(CommandResult::success(output))
}

fn verify_plan(
    options: &MirrorsOptions,
    plan_path: &std::path::Path,
) -> Result<CommandResult, CliError> {
    let plan_path = repository_file(&options.root, plan_path).map_err(CliError::new)?;
    let source = read_text(&plan_path)
        .map_err(|error| CliError::new(format!("read mirror plan: {error}")))?;
    let verification = verify_test_mirrors(&options.root, &options.config, &source)
        .map_err(|error| CliError::new(error.to_string()))?;
    let output = match options.format {
        OutputFormat::Human => verification.human(),
        OutputFormat::Json => verification
            .json()
            .map_err(|error| CliError::new(format!("serialize mirror verification: {error}")))?,
    };
    let exit_code = i32::from(verification.report.status != ReportStatus::Pass);
    Ok(CommandResult::status(output, exit_code))
}
