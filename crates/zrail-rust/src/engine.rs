//! Public orchestration for checking, locking, and diagnosing a Rust repository.

mod lock_compare;
mod lock_state;
mod model;

use std::{error::Error, fmt, path::Path};

use serde::{Deserialize, Serialize};
use zrail_core::{LockFile, Report};

use crate::rules::{RuleContext, evaluate};

pub(crate) use self::model::load_model;

use self::{
    lock_compare::{check_lock, requires_lock},
    lock_state::{candidate_lock, read_optional_lock},
    model::resolve,
};

#[derive(Clone, Debug)]
pub struct CheckResult {
    pub report: Report,
    pub candidate_lock: LockFile,
    pub contract_sha256: String,
    pub packages: usize,
    pub rust_files: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DoctorReport {
    pub schema: u64,
    pub root: String,
    pub config: String,
    pub lock: String,
    pub contract_sha256: String,
    pub packages: usize,
    pub rust_files: usize,
    pub contract_sources: usize,
    pub status: String,
}

impl DoctorReport {
    pub fn json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self).map(|mut value| {
            value.push('\n');
            value
        })
    }

    pub fn is_ready(&self) -> bool {
        self.status == "ready"
    }

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

pub fn check_repository(
    root: &Path,
    config: &Path,
    lock_path: &Path,
) -> Result<CheckResult, CheckError> {
    let model = load_model(root, config)?;
    let lock = read_optional_lock(&model.inventory.root, lock_path)?;
    check_model(model, lock.as_ref())
}

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

pub fn build_lock(root: &Path, config: &Path) -> Result<LockFile, CheckError> {
    candidate_lock(&load_model(root, config)?)
}

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
    if current.engine != env!("CARGO_PKG_VERSION") {
        return "lock-engine-mismatch";
    }
    if current != candidate {
        return "lock-stale";
    }
    "ready"
}

#[cfg(test)]
#[path = "engine_test.rs"]
mod engine_test;
