//! Protected source review derives proposed architecture without writing it.

use zrail_core::{
    Finding, LOCK_SCHEMA, LOCK_SEMANTICS, LockFile, Report, ReportStatus, load_contract,
    repository_file,
};
use zrail_rust::check_repository_with_limit;

use crate::app::{args::ReviewOptions, error::CliError, output::OutputFormat};

use super::{CommandResult, update_authority};

pub(crate) fn review(options: &ReviewOptions) -> Result<CommandResult, CliError> {
    let common = &options.common;
    let checked =
        check_repository_with_limit(&common.root, &common.config, &common.lock, common.limit)
            .map_err(|error| CliError::new(error.to_string()))?;
    let Some(candidate) = checked.candidate_lock.clone() else {
        let text = match common.format {
            OutputFormat::Human => checked.report.human(),
            OutputFormat::Json => checked
                .report
                .json()
                .map_err(|error| CliError::new(format!("serialize source review: {error}")))?,
        };
        return Ok(CommandResult::status(text, 2));
    };
    let proposed = load_contract(&common.root, &common.config)
        .map_err(|error| CliError::new(format!("load proposed contract: {error}")))?;
    let architecture = update_authority::compare_from_repository(
        &options.authority_root,
        &options.base,
        &common.config,
        &common.lock,
        &proposed,
        &candidate,
        None,
        None,
    )?;
    let source = checked
        .report
        .with_findings(lock_attestation(options, &candidate)?);
    let failed = source.status != ReportStatus::Pass
        || architecture_denied(&architecture, options.allow_grants);
    let text = match common.format {
        OutputFormat::Human => human(&source, &architecture),
        OutputFormat::Json => json(&source, &architecture, failed)?,
    };
    Ok(CommandResult::status(text, i32::from(failed)))
}

fn architecture_denied(report: &zrail_core::DiffReport, allow_grants: bool) -> bool {
    report.summary.debt > 0
        || report.summary.unknown > 0
        || (!allow_grants && report.summary.grants > 0)
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
    if !proposed.has_supported_schema() {
        return Ok(vec![
            Finding::error(
                "REVIEW-003",
                "review.lock",
                "review",
                format!(
                    "proposed zrail.lock uses schema {}, latest supported schema is {}",
                    proposed.schema, LOCK_SCHEMA
                ),
            )
            .with_help("use a protected zrail engine that understands the proposed lock schema"),
        ]);
    }
    if !proposed.has_current_semantics() {
        return Ok(vec![
            Finding::error(
                "REVIEW-004",
                "review.lock",
                "review",
                format!(
                    "proposed zrail.lock uses semantics {}, current semantics are {}",
                    proposed.semantics, LOCK_SEMANTICS
                ),
            )
            .with_help("regenerate the proposed lock with the protected zrail engine"),
        ]);
    }
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
