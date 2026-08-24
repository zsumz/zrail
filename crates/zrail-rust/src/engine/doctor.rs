//! Repository and lock readiness reporting.

use std::path::Path;

use serde::{Deserialize, Serialize};
use zrail_core::LockFile;

use crate::analysis::AnalysisOutcome;

use super::{
    CheckError,
    lock_compare::requires_lock,
    lock_state::{candidate_lock, read_optional_lock},
    model::{load_model, resolve},
};

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// Machine-readable readiness information for a repository and its lock.
pub struct DoctorReport {
    /// The schema version of this serialized doctor report.
    pub schema: u64,
    /// The analyzed repository's resolved root path.
    pub root: String,
    /// The resolved path to the contract's entry configuration file.
    pub config: String,
    /// The resolved path where the repository lock is expected.
    pub lock: String,
    /// The SHA-256 digest of the complete resolved contract bundle.
    pub contract_sha256: String,
    /// The number of Cargo packages included in the analysis.
    pub packages: usize,
    /// The number of Rust source files included in the analysis.
    pub rust_files: usize,
    /// The number of files that contribute to the resolved contract bundle.
    pub contract_sources: usize,
    /// Explicit repository-wide analysis completeness and work census.
    pub analysis: AnalysisOutcome,
    /// Lock readiness: `ready`, `lock-missing`, `lock-schema-mismatch`,
    /// `lock-semantics-mismatch`, and `lock-stale`.
    pub status: String,
}

impl DoctorReport {
    /// Serializes the report as pretty JSON terminated by a newline.
    pub fn json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self).map(|mut value| {
            value.push('\n');
            value
        })
    }

    /// Returns whether the repository has no lock-readiness problem.
    pub fn is_ready(&self) -> bool {
        self.status == "ready"
    }

    /// Renders a concise multiline report for a terminal.
    pub fn human(&self) -> String {
        format!(
            concat!(
                "zrail doctor\n\n",
                "root: {}\n",
                "config: {}\n",
                "lock: {}\n",
                "contract: {}\n",
                "packages: {}\n",
                "rust files: {}\n",
                "contract sources: {}\n",
                "analysis: {}\n",
                "status: {}\n",
            ),
            self.root,
            self.config,
            self.lock,
            self.contract_sha256,
            self.packages,
            self.rust_files,
            self.contract_sources,
            if self.analysis.is_complete() {
                "complete"
            } else {
                "incomplete"
            },
            self.status
        )
    }
}

/// Reports whether a repository's configured lock is present, supported, and current.
///
/// `config` and `lock` may be relative to `root`; resolved paths must remain within
/// the repository. The operation is read-only.
pub fn doctor_repository(
    root: &Path,
    config: &Path,
    lock: &Path,
) -> Result<DoctorReport, CheckError> {
    let model = load_model(root, config)?;
    let analysis = AnalysisOutcome::from_source(&model.source);
    let current = read_optional_lock(&model.inventory.root, lock)?;
    let lock_path = resolve(&model.inventory.root, lock)?;
    let status = if analysis.is_complete() {
        let candidate = candidate_lock(&model)?;
        doctor_status(
            requires_lock(&model.bundle.contract),
            current.as_ref(),
            &candidate,
        )
    } else {
        "analysis-incomplete"
    };
    Ok(DoctorReport {
        schema: 1,
        root: model.inventory.root.to_string_lossy().into_owned(),
        config: resolve(&model.inventory.root, config)?
            .to_string_lossy()
            .into_owned(),
        lock: lock_path.to_string_lossy().into_owned(),
        contract_sha256: model.bundle.sha256,
        packages: model.cargo.packages.len(),
        rust_files: model.source.files.len(),
        contract_sources: model.bundle.sources.len(),
        analysis,
        status: status.into(),
    })
}

fn doctor_status(
    lock_required: bool,
    current: Option<&LockFile>,
    candidate: &LockFile,
) -> &'static str {
    if !lock_required {
        return "ready";
    }
    let Some(current) = current else {
        return "lock-missing";
    };
    if !current.has_supported_schema() {
        return "lock-schema-mismatch";
    }
    if !current.has_current_semantics() {
        return "lock-semantics-mismatch";
    }
    if !current.same_resolved_state(candidate) {
        return "lock-stale";
    }
    "ready"
}

#[cfg(test)]
#[path = "doctor_test.rs"]
mod doctor_test;
