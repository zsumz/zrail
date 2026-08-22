//! Candidate state supports lock updates and non-mutating contract proposals.

use std::path::Path;

use zrail_core::{ContractBundle, DiagnosticLimit, LockFile};

use super::{
    CheckError, CheckResult, finish_check,
    lock_state::candidate_lock,
    model::{load_model, load_model_with_bundle},
};

/// Builds the lock state observed from a repository and contract.
///
/// The returned lock remains in memory. This function neither writes a lock file
/// nor executes repository code, Cargo, build scripts, or qualification gates.
pub fn build_lock(root: &Path, config: &Path) -> Result<LockFile, CheckError> {
    candidate_lock(&load_model(root, config)?)
}

/// Checks a repository against an in-memory validated contract and its candidate lock.
///
/// The bundle's source identities and digest remain authoritative. This form is
/// intended for proposal workflows that must prove a contract edit before any
/// repository file is replaced.
pub fn check_repository_with_candidate_contract(
    root: &Path,
    bundle: ContractBundle,
) -> Result<CheckResult, CheckError> {
    let model = load_model_with_bundle(root, bundle)?;
    let candidate = candidate_lock(&model)?;
    let accepted = candidate.clone();
    Ok(finish_check(
        model,
        Some(&accepted),
        candidate,
        DiagnosticLimit::default(),
    ))
}
