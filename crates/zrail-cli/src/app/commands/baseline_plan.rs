//! One non-mutating planner serves hand-authored and initialized contracts.

use std::{fs, path::Path};

use zrail_core::{
    ContractBundle, LockFile, Report, load_contract, load_contract_with_entry, read_text,
    repository_file,
};
use zrail_rust::{
    BaselineRatchet, BaselineRule, check_repository_with_candidate_contract,
    discover_baseline_rules,
};

use crate::app::error::CliError;

use super::baseline_edit;

#[derive(Debug)]
pub(super) struct PreparedBaseline {
    pub(super) root: std::path::PathBuf,
    pub(super) config_path: std::path::PathBuf,
    pub(super) original_contract: String,
    pub(super) patched_contract: String,
    pub(super) before: ContractBundle,
    pub(super) after: ContractBundle,
    pub(super) candidate_lock: LockFile,
    pub(super) report: Report,
    pub(super) added: Vec<BaselineRatchet>,
    pub(super) preserved: Vec<BaselineRatchet>,
}

pub(super) fn prepare(
    root: &Path,
    config: &Path,
    selected: Option<&str>,
) -> Result<PreparedBaseline, CliError> {
    let root = fs::canonicalize(root)
        .map_err(|error| CliError::new(format!("open repository {}: {error}", root.display())))?;
    let config_path = repository_file(&root, config).map_err(CliError::new)?;
    let original_contract = read_text(&config_path).map_err(CliError::new)?;
    let before = load_contract(&root, config)
        .map_err(|error| CliError::new(format!("load existing contract: {error}")))?;
    let rules = selected_rules(selected)?;
    let discovered = discover_baseline_rules(&root, config, &rules)
        .map_err(|error| CliError::new(error.to_string()))?;
    let edit = baseline_edit::merge(
        &original_contract,
        &before.contract.ratchets,
        discovered.ratchets,
    );
    let after = load_contract_with_entry(&root, config, &edit.contract)
        .map_err(|error| CliError::new(format!("load baseline proposal: {error}")))?;
    let checked = check_repository_with_candidate_contract(&root, after.clone())
        .map_err(|error| CliError::new(error.to_string()))?;
    let candidate_lock = checked.candidate_lock.ok_or_else(|| {
        let causes = checked
            .analysis
            .issues()
            .iter()
            .map(|issue| format!("{}: {}", issue.id, issue.message))
            .collect::<Vec<_>>()
            .join("; ");
        CliError::new(format!(
            "baseline cannot adopt incomplete analysis: {causes}"
        ))
    })?;
    Ok(PreparedBaseline {
        root,
        config_path,
        original_contract,
        patched_contract: edit.contract,
        before,
        after,
        candidate_lock,
        report: checked.report,
        added: edit.added,
        preserved: edit.preserved,
    })
}

fn selected_rules(selected: Option<&str>) -> Result<Vec<BaselineRule>, CliError> {
    match selected {
        Some(name) => BaselineRule::named(name).map_or_else(
            || {
                Err(CliError::new(format!(
                    "unsupported baseline rule {name:?}; expected one of {}",
                    BaselineRule::ALL
                        .iter()
                        .map(|rule| rule.name())
                        .collect::<Vec<_>>()
                        .join(", ")
                )))
            },
            |rule| Ok(vec![rule]),
        ),
        None => Ok(BaselineRule::ALL.to_vec()),
    }
}
