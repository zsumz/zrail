//! Complete, deterministic governed-surface reporting for audit consumers.

mod dependencies;
mod model;
mod output;
mod owners;
mod rails;

use std::path::Path;

use zrail_core::AnalysisQuality;

use crate::{
    analysis::AnalysisOutcome,
    engine::{CheckError, load_model},
};

pub use model::{
    GovernedAnalysis, GovernedCompilationDomain, GovernedDependencyPath, GovernedDependencyRule,
    GovernedOperationOccurrence, GovernedOwnerRule, GovernedPackageIdentity, GovernedSurfaceReport,
    GovernedTestMirror,
};

const REPORT_SCHEMA: u64 = 3;

/// Builds a read-only audit report for every governed source and dependency surface.
///
/// Relative configuration paths are interpreted beneath `root`. The operation
/// fails instead of returning partial evidence when source analysis or exact
/// dependency resolution is incomplete.
pub fn governed_surface_report(
    root: &Path,
    config: &Path,
) -> Result<GovernedSurfaceReport, CheckError> {
    let model = load_model(root, config)?;
    let outcome = AnalysisOutcome::from_source(&model.source);
    if !outcome.is_complete() {
        return Err(incomplete_error(&outcome));
    }
    let owners = owners::report(&model);
    let dependencies = dependencies::report(&model).map_err(CheckError::from_message)?;
    let mut exclusions = model.bundle.contract.repository.exclude.clone();
    exclusions.sort();
    exclusions.dedup();
    let mut test_mirrors = model
        .bundle
        .contract
        .source
        .rust
        .test_mirrors
        .iter()
        .map(|mirror| GovernedTestMirror {
            policy_id: format!("test-mirror:{}::{}", mirror.test, mirror.name),
            production: mirror.production.clone(),
            test: mirror.test.clone(),
            test_name: mirror.name.clone(),
            receipt: mirror.receipt.clone(),
            inputs: mirror.inputs.clone(),
            command: mirror.execution.command.clone(),
            package: mirror.execution.package.clone(),
            default_features: mirror.execution.default_features,
            features: mirror.execution.features.clone(),
            target: mirror.execution.target.clone(),
            toolchain: mirror.execution.toolchain.clone(),
            reason: mirror.reason.clone(),
        })
        .collect::<Vec<_>>();
    test_mirrors.sort_by(|left, right| left.policy_id.cmp(&right.policy_id));
    let enabled_rails = rails::report(&model, &owners, &dependencies, &test_mirrors);
    let occurrences = owners.iter().flat_map(|owner| &owner.occurrences);
    let unresolved_occurrences = occurrences
        .clone()
        .filter(|occurrence| occurrence.quality == AnalysisQuality::Unresolved)
        .count();
    let ambiguous_occurrences = occurrences
        .filter(|occurrence| occurrence.quality == AnalysisQuality::Conservative)
        .count();
    Ok(GovernedSurfaceReport {
        schema: REPORT_SCHEMA,
        contract_schema: model.bundle.contract.schema,
        contract_sha256: model.bundle.sha256.clone(),
        analysis: GovernedAnalysis {
            complete: true,
            metrics: outcome.metrics(),
            exclusions,
        },
        unresolved_occurrences,
        ambiguous_occurrences,
        enabled_rails,
        owners,
        dependencies,
        test_mirrors,
    })
}

fn incomplete_error(outcome: &AnalysisOutcome) -> CheckError {
    let mut issues = outcome
        .issues()
        .iter()
        .map(|issue| {
            format!(
                "{}@{}",
                issue.id,
                issue.path.as_deref().unwrap_or("repository")
            )
        })
        .collect::<Vec<_>>();
    issues.sort();
    CheckError::from_message(format!(
        "coverage requires complete analysis; {} unresolved issue(s): {}",
        issues.len(),
        issues.join(", ")
    ))
}

#[cfg(test)]
#[path = "coverage_test.rs"]
mod coverage_test;
