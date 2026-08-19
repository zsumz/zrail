//! Public orchestration for checking, locking, and diagnosing a Rust repository.

mod gates;
mod lock_compare;
mod lock_state;
mod macro_implementations;
mod model;

use std::{error::Error, fmt, path::Path};

use serde::{Deserialize, Serialize};
use zrail_core::{LockFile, Report};

use crate::rules::{RuleContext, evaluate};

pub(crate) use self::model::{RepositoryModel, load_model};

use self::{
    lock_compare::{check_lock, requires_lock},
    lock_state::{candidate_lock, read_optional_lock},
    model::resolve,
};

#[derive(Clone, Debug)]
/// The diagnostics and independently observed state from one repository check.
pub struct CheckResult {
    /// The deterministic findings and aggregate report status.
    pub report: Report,
    /// The lock state derived from the current contract and repository contents.
    ///
    /// This value is returned in memory; checking does not write it to disk.
    pub candidate_lock: LockFile,
    /// The SHA-256 digest of the complete resolved contract bundle.
    pub contract_sha256: String,
    /// The number of Cargo packages included in the analysis.
    pub packages: usize,
    /// The number of Rust source files included in the analysis.
    pub rust_files: usize,
}

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
                "status: {}\n",
            ),
            self.root,
            self.config,
            self.lock,
            self.contract_sha256,
            self.packages,
            self.rust_files,
            self.contract_sources,
            self.status
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// An input, repository-boundary, parsing, or analysis error.
///
/// The message is intended for display. Policy violations are returned as
/// findings in [`CheckResult::report`] rather than as this error type.
pub struct CheckError(String);

impl CheckError {
    pub(crate) fn from_message(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for CheckError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for CheckError {}

#[doc = include_str!("check_repository.md")]
pub fn check_repository(
    root: &Path,
    config: &Path,
    lock_path: &Path,
) -> Result<CheckResult, CheckError> {
    let model = load_model(root, config)?;
    let lock = read_optional_lock(&model.inventory.root, lock_path)?;
    check_model(model, lock.as_ref())
}

/// Checks a repository against an already loaded lock without writing files.
///
/// `config` may be relative to `root` and must resolve within the repository. This
/// form is useful when the caller obtained a lock through a separately authorized
/// workflow instead of reading the repository's lock path.
pub fn check_repository_with_lock(
    root: &Path,
    config: &Path,
    lock: &LockFile,
) -> Result<CheckResult, CheckError> {
    check_model(load_model(root, config)?, Some(lock))
}

fn check_model(
    model: model::RepositoryModel,
    lock: Option<&LockFile>,
) -> Result<CheckResult, CheckError> {
    let candidate = candidate_lock(&model)?;
    let mut findings = evaluate(&RuleContext {
        contract: &model.bundle.contract,
        lock,
        inventory: &model.inventory,
        cargo: &model.cargo,
        source: &model.source,
    });
    check_lock(&model, lock, &candidate, &mut findings);
    Ok(CheckResult {
        report: Report::from_findings(findings.into_findings()),
        candidate_lock: candidate,
        contract_sha256: model.bundle.sha256,
        packages: model.cargo.packages.len(),
        rust_files: model.source.files.len(),
    })
}

/// Builds the lock state observed from a repository and contract.
///
/// The returned lock remains in memory. This function neither writes a lock file
/// nor executes repository code, Cargo, build scripts, or qualification gates.
pub fn build_lock(root: &Path, config: &Path) -> Result<LockFile, CheckError> {
    candidate_lock(&load_model(root, config)?)
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
    let candidate = candidate_lock(&model)?;
    let current = read_optional_lock(&model.inventory.root, lock)?;
    let lock_path = resolve(&model.inventory.root, lock)?;
    let status = doctor_status(
        requires_lock(&model.bundle.contract),
        current.as_ref(),
        &candidate,
    );
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
#[path = "engine_test.rs"]
mod engine_test;
