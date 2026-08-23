//! Public orchestration for checking, locking, and diagnosing a Rust repository.

mod candidate;
mod doctor;
mod gates;
mod lock_compare;
mod lock_state;
mod macro_implementations;
mod model;

use std::{error::Error, fmt, path::Path};

use zrail_core::{DiagnosticLimit, LockFile, Report};

use crate::rules::{RuleContext, evaluate};

pub(crate) use self::model::{RepositoryModel, load_model};

use self::{
    lock_compare::check_lock,
    lock_state::{candidate_lock, read_optional_lock},
};

pub use candidate::{build_lock, check_repository_with_candidate_contract};
pub use doctor::{DoctorReport, doctor_repository};

#[derive(Clone, Debug)]
/// The diagnostics and independently observed state from one repository check.
pub struct CheckResult {
    /// The deterministic findings and aggregate report status.
    pub report: Report,
    /// Current contract and repository state, returned without writing it to disk.
    pub candidate_lock: LockFile,
    /// The SHA-256 digest of the complete resolved contract bundle.
    pub contract_sha256: String,
    /// The number of Cargo packages included in the analysis.
    pub packages: usize,
    /// The number of Rust source files included in the analysis.
    pub rust_files: usize,
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
    check_model(model, lock.as_ref(), DiagnosticLimit::default())
}

/// Checks a repository with an explicit individual-diagnostic payload limit.
///
/// Every finding contributes to exact status and aggregate counts even when its
/// individual payload is omitted. The operation remains read-only.
pub fn check_repository_with_limit(
    root: &Path,
    config: &Path,
    lock_path: &Path,
    limit: DiagnosticLimit,
) -> Result<CheckResult, CheckError> {
    let model = load_model(root, config)?;
    let lock = read_optional_lock(&model.inventory.root, lock_path)?;
    check_model(model, lock.as_ref(), limit)
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
    check_model(
        load_model(root, config)?,
        Some(lock),
        DiagnosticLimit::default(),
    )
}

fn check_model(
    model: model::RepositoryModel,
    lock: Option<&LockFile>,
    limit: DiagnosticLimit,
) -> Result<CheckResult, CheckError> {
    let candidate = candidate_lock(&model)?;
    Ok(finish_check(model, lock, candidate, limit))
}

fn finish_check(
    model: model::RepositoryModel,
    lock: Option<&LockFile>,
    candidate: LockFile,
    limit: DiagnosticLimit,
) -> CheckResult {
    let mut findings = evaluate(
        &RuleContext {
            contract: &model.bundle.contract,
            lock,
            inventory: &model.inventory,
            cargo: &model.cargo,
            source: &model.source,
            module_edges: &model.module_edges,
        },
        limit,
    );
    check_lock(&model, lock, &candidate, &mut findings);
    CheckResult {
        report: Report::from_sink(findings),
        candidate_lock: candidate,
        contract_sha256: model.bundle.sha256,
        packages: model.cargo.packages.len(),
        rust_files: model.source.files.len(),
    }
}
