//! Candidate state supports lock updates and non-mutating contract proposals.

use std::path::Path;

use zrail_core::{ContractBundle, DiagnosticLimit, LockFile};

use crate::analysis::AnalysisOutcome;

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
    let model = load_model(root, config)?;
    require_complete(&model)?;
    candidate_lock(&model)
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
    let analysis = AnalysisOutcome::from_source(&model.source);
    let candidate = analysis
        .is_complete()
        .then(|| candidate_lock(&model))
        .transpose()?;
    let accepted = candidate.clone();
    Ok(finish_check(
        model,
        accepted.as_ref(),
        candidate,
        analysis,
        DiagnosticLimit::default(),
    ))
}

fn require_complete(model: &super::model::RepositoryModel) -> Result<(), CheckError> {
    let analysis = AnalysisOutcome::from_source(&model.source);
    if analysis.is_complete() {
        return Ok(());
    }
    let causes = analysis
        .issues()
        .iter()
        .map(|issue| format!("{}: {}", issue.id, issue.message))
        .collect::<Vec<_>>()
        .join("; ");
    Err(CheckError::from_message(format!(
        "refusing to construct zrail.lock from incomplete analysis: {causes}"
    )))
}
