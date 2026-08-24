//! Lock-migration artifacts embed the core report without a CLI serialization dependency.

use zrail_core::LockMigrationReport;

use crate::app::{error::CliError, output::json_escape};

pub(super) fn render(
    base_commit: &str,
    contract_sha256: &str,
    report_sha256: &str,
    report: &LockMigrationReport,
) -> Result<String, CliError> {
    let report = report
        .json()
        .map_err(|error| CliError::new(format!("serialize lock migration report: {error}")))?;
    let report = report.trim_end().replace('\n', "\n  ");
    Ok(format!(
        concat!(
            "{{\n",
            "  \"schema\": 1,\n",
            "  \"base_commit\": \"{}\",\n",
            "  \"contract_sha256\": \"{}\",\n",
            "  \"report_sha256\": \"{}\",\n",
            "  \"report\": {}\n",
            "}}\n",
        ),
        json_escape(base_commit),
        json_escape(contract_sha256),
        json_escape(report_sha256),
        report,
    ))
}

#[cfg(test)]
#[path = "migrate_lock_artifact_test.rs"]
mod migrate_lock_artifact_test;
