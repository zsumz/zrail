//! Closed inventories reuse parsed facts without changing ownership resolution or executing code.

mod model;
mod observe;
mod output;

use zrail_core::{
    AnalysisQuality, RepositoryEntryMode, RustInventoryAssertion, RustInventoryRule, sha256_hex,
};

use crate::{inventory::RepositoryInventory, repository_files::Selection, source::SourceIndex};

pub(crate) use model::RustInventoryAnalysis;
pub use model::{GovernedRustInventory, RustInventoryInput, RustInventoryOccurrence};
pub(crate) use output::{display, evaluate, for_path};

pub(crate) fn analyze(
    inventory: &RepositoryInventory,
    source: &SourceIndex,
    rules: &[RustInventoryRule],
) -> Result<RustInventoryAnalysis, String> {
    let mut analysis = RustInventoryAnalysis::default();
    if rules.is_empty() {
        return Ok(analysis);
    }
    let root = &inventory.root;
    let mut selection = Selection::new(
        root,
        rules
            .iter()
            .flat_map(|rule| rule.include.iter().map(String::as_str)),
    )
    .map_err(|error| incomplete("rust:inventories", error))?;
    let mut observations = observe::Observations::new(inventory, source);
    let mut rules = rules.to_vec();
    rules.sort_by(|left, right| left.name.cmp(&right.name));
    for mut policy in rules {
        let policy_id = format!("rust:inventory:{}", policy.name);
        let context_error = |error| incomplete(&policy_id, error);
        policy.include.sort();
        policy.exclude.sort();
        policy.subject.canonicalize();
        match &mut policy.assertion {
            RustInventoryAssertion::Count { .. } => {}
            RustInventoryAssertion::ExactCounts { counts } => counts.sort(),
            RustInventoryAssertion::ExactOwners { owners } => owners.sort(),
        }
        let selected = selection
            .select(
                root,
                &policy.include,
                &policy.exclude,
                RepositoryEntryMode::File,
            )
            .map_err(&context_error)?;
        let mut paths = Vec::new();
        for entry in selected {
            if std::path::Path::new(&entry.path).extension() != Some(std::ffi::OsStr::new("rs"))
                || entry.resolved_path.is_some()
            {
                return Err(context_error(format!(
                    "selected path {:?} must be a physical Rust file, not a link or non-Rust input",
                    entry.path
                )));
            }
            paths.push(entry.path);
        }
        let mut report = GovernedRustInventory {
            policy_id: policy_id.clone(),
            claim: match policy.subject {
                zrail_core::RustInventorySubject::WrittenMethods { .. } => {
                    "authored-rust-method-call-syntax".into()
                }
                zrail_core::RustInventorySubject::WrittenExpressionPaths { .. } => {
                    "authored-rust-expression-path-syntax".into()
                }
                zrail_core::RustInventorySubject::WrittenPathsContaining { .. } => {
                    "authored-rust-path-membership-syntax".into()
                }
                zrail_core::RustInventorySubject::WrittenImportRenames { .. } => {
                    "authored-rust-import-rename-syntax".into()
                }
            },
            policy,
            quality: AnalysisQuality::Exact,
            inputs: Vec::new(),
            counts: Vec::new(),
            observed_count: 0,
            occurrence_sample: Vec::new(),
            occurrences_omitted: 0,
            satisfied: false,
        };
        observations
            .populate(&mut report, &paths)
            .map_err(context_error)?;
        report.satisfied = match &report.policy.assertion {
            RustInventoryAssertion::Count { minimum, maximum } => {
                report.observed_count >= *minimum
                    && maximum.is_none_or(|max| report.observed_count <= max)
            }
            RustInventoryAssertion::ExactCounts { counts } => *counts == report.counts,
            RustInventoryAssertion::ExactOwners { owners } => owners
                .iter()
                .map(|owner| (&owner.path, &owner.name))
                .eq(report.counts.iter().map(|count| (&count.path, &count.name))),
        };
        analysis.policies.push(report);
    }
    let bytes = serde_json::to_vec(&analysis.policies)
        .map_err(|error| incomplete("rust:inventories", error.to_string()))?;
    analysis.binding_sha256 = Some(sha256_hex(&bytes));
    Ok(analysis)
}

fn incomplete(policy: &str, error: impl std::fmt::Display) -> String {
    format!("RUST-INVENTORY-002: incomplete Rust inventory {policy}: {error}")
}

#[cfg(test)]
#[path = "rust_inventories_test.rs"]
mod rust_inventories_test;
