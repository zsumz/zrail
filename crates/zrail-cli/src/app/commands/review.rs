//! Protected source review derives proposed architecture without writing it.

use zrail_core::{Finding, LockFile, Report, ReportStatus, load_contract, path::repository_file};
use zrail_rust::check_repository;

use crate::app::{args::ReviewOptions, error::CliError, output::OutputFormat};

use super::{CommandResult, update_authority};

pub(crate) fn review(options: &ReviewOptions) -> Result<CommandResult, CliError> {
    let common = &options.common;
    let checked = check_repository(&common.root, &common.config, &common.lock)
        .map_err(|error| CliError::new(error.to_string()))?;
    let proposed = load_contract(&common.root, &common.config)
        .map_err(|error| CliError::new(format!("load proposed contract: {error}")))?;
    let architecture = update_authority::compare_from_repository(
        &options.authority_root,
        &options.base,
        &common.config,
        &common.lock,
        &proposed,
        &checked.candidate_lock,
    )?;
    let mut findings = checked.report.findings;
    findings.extend(lock_attestation(options, &checked.candidate_lock)?);
    let source = Report::from_findings(findings);
    let failed = source.status != ReportStatus::Pass
        || (options.deny_grants && architecture.denies_grants());
    let text = match common.format {
        OutputFormat::Human => human(&source, &architecture),
        OutputFormat::Json => json(&source, &architecture, failed)?,
    };
    Ok(CommandResult::status(text, i32::from(failed)))
}

fn lock_attestation(
    options: &ReviewOptions,
    candidate: &LockFile,
) -> Result<Vec<Finding>, CliError> {
    let path =
        repository_file(&options.common.root, &options.common.lock).map_err(CliError::new)?;
    let proposed = LockFile::read_optional(&path)
        .map_err(|error| CliError::new(format!("load proposed lock: {error}")))?;
    let Some(proposed) = proposed else {
        return Ok(vec![
            Finding::error(
                "REVIEW-001",
                "review.lock",
                "review",
                "protected review requires a checked-in proposed zrail.lock",
            )
            .with_help("generate and review the proposed lock before protected review"),
        ]);
    };
    if proposed.same_resolved_state(candidate) {
        Ok(Vec::new())
    } else {
        Ok(vec![
            Finding::error(
                "REVIEW-002",
                "review.lock",
                "review",
                "proposed zrail.lock does not match independently observed repository state",
            )
            .with_help("regenerate the lock, then review its semantic architecture diff"),
        ])
    }
}

fn human(source: &Report, architecture: &zrail_core::DiffReport) -> String {
    format!(
        "Proposed source\n\n{}\nArchitecture authority\n\n{}",
        source.human(),
        architecture.human()
    )
}

fn json(
    source: &Report,
    architecture: &zrail_core::DiffReport,
    failed: bool,
) -> Result<String, CliError> {
    let source = source
        .json()
        .map_err(|error| CliError::new(format!("serialize source review: {error}")))?;
    let architecture = architecture
        .json()
        .map_err(|error| CliError::new(format!("serialize architecture review: {error}")))?;
    Ok(format!(
        concat!(
            "{{\n",
            "  \"schema\": 1,\n",
            "  \"status\": \"{}\",\n",
            "  \"source\": {},\n",
            "  \"architecture\": {}\n",
            "}}\n",
        ),
        if failed { "fail" } else { "pass" },
        source.trim(),
        architecture.trim(),
    ))
}

#[cfg(test)]
#[path = "review_test.rs"]
mod review_test;
