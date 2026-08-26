//! Deterministic, non-executing plans for exact test-mirror evidence.

mod model;
mod output;
mod receipts;

use std::{collections::BTreeMap, path::Path};

use zrail_core::{FindingSink, Report, TestMirrorContract};

use crate::{
    analysis::AnalysisOutcome,
    engine::{CheckError, RepositoryModel, load_model},
    inventory::{FileClass, RepositoryEntry},
    mirror_inputs::MirrorInputCache,
    source::{ReachabilityKind, RustFileFacts},
};

pub(crate) use model::policy_id as test_mirror_policy_id;
pub use model::{MirrorPlan, MirrorVerification, PlannedTestMirror};
pub use receipts::{
    MirrorExecutionResult, MirrorReceiptBundle, MirrorResultSet, MirrorTestResult,
    RenderedMirrorReceipt,
};

/// Builds the exact expected mirror-receipt plan without executing repository code.
pub fn test_mirror_plan(root: &Path, config: &Path) -> Result<MirrorPlan, CheckError> {
    load_plan(root, config).map(|(_, plan)| plan)
}

fn load_plan(root: &Path, config: &Path) -> Result<(RepositoryModel, MirrorPlan), CheckError> {
    let model = load_model(root, config)?;
    let outcome = AnalysisOutcome::from_source(&model.source);
    if !outcome.is_complete() {
        return Err(CheckError::from_message(format!(
            "mirror planning requires complete analysis; {} unresolved issue(s)",
            outcome.issues().len()
        )));
    }
    let plan = plan(&model, outcome.metrics())?;
    Ok((model, plan))
}

/// Recomputes a mirror plan and rejects stale, malformed, or mismatched plan JSON.
pub fn verify_test_mirror_plan(
    root: &Path,
    config: &Path,
    plan_source: &str,
) -> Result<MirrorPlan, CheckError> {
    let claimed = MirrorPlan::parse(plan_source).map_err(CheckError::from_message)?;
    let (_, observed) = load_plan(root, config)?;
    require_current_plan(&claimed, &observed)?;
    Ok(observed)
}

/// Verifies a current plan and every declared schema-2 receipt without executing tests.
pub fn verify_test_mirrors(
    root: &Path,
    config: &Path,
    plan_source: &str,
) -> Result<MirrorVerification, CheckError> {
    let claimed = MirrorPlan::parse(plan_source).map_err(CheckError::from_message)?;
    let (model, plan) = load_plan(root, config)?;
    require_current_plan(&claimed, &plan)?;
    let context = crate::rules::RuleContext {
        contract: &model.bundle.contract,
        lock: None,
        inventory: &model.inventory,
        cargo: &model.cargo,
        resolved_cargo: model.resolved_cargo.as_ref(),
        source: &model.source,
        module_edges: &model.module_edges,
        compilation_domains: &model.compilation_domains,
        feature_worlds: &model.feature_worlds,
    };
    let mut findings = FindingSink::default();
    crate::rules::evidence::check_mirrors(&context, &mut findings);
    Ok(MirrorVerification {
        schema: 1,
        plan_sha256: plan.plan_sha256,
        mirrors: plan.mirrors.len(),
        report: Report::from_sink(findings),
    })
}

fn require_current_plan(claimed: &MirrorPlan, observed: &MirrorPlan) -> Result<(), CheckError> {
    if claimed != observed {
        return Err(CheckError::from_message(
            "mirror plan differs from the current exact contract and inputs",
        ));
    }
    Ok(())
}

/// Renders every schema-2 receipt from a current plan and strict trusted result set.
///
/// This validates data only. It does not execute tests or write receipt files.
pub fn render_test_mirror_receipts(
    root: &Path,
    config: &Path,
    plan_source: &str,
    result_source: &str,
) -> Result<MirrorReceiptBundle, CheckError> {
    let plan = verify_test_mirror_plan(root, config, plan_source)?;
    let results = MirrorResultSet::parse(result_source).map_err(CheckError::from_message)?;
    MirrorReceiptBundle::render(&plan, results).map_err(CheckError::from_message)
}

fn plan(
    model: &RepositoryModel,
    metrics: crate::AnalysisMetrics,
) -> Result<MirrorPlan, CheckError> {
    let entries = model
        .inventory
        .entries
        .iter()
        .map(|entry| (entry.relative.as_str(), entry))
        .collect::<BTreeMap<&str, &RepositoryEntry>>();
    let mut cache = MirrorInputCache::new(&entries);
    let mut mirrors = Vec::new();
    for mirror in &model.bundle.contract.source.rust.test_mirrors {
        validate_mirror(model, mirror)?;
        let input_sha256 = cache.digest(mirror).map_err(CheckError::from_message)?;
        mirrors
            .push(PlannedTestMirror::new(mirror, input_sha256).map_err(CheckError::from_message)?);
    }
    mirrors.sort_by(|left, right| left.policy_id.cmp(&right.policy_id));
    MirrorPlan::new(model.bundle.sha256.clone(), metrics, mirrors).map_err(CheckError::from_message)
}

fn validate_mirror(model: &RepositoryModel, mirror: &TestMirrorContract) -> Result<(), CheckError> {
    let production = exact_file(model, &mirror.production, "production")?;
    if !production.reachability.is_production() {
        return Err(CheckError::from_message(format!(
            "mirror production {:?} is not production reachable",
            mirror.production
        )));
    }
    let test = exact_file(model, &mirror.test, "test")?;
    if test.class != FileClass::Test || !test.reachability.contains(ReachabilityKind::Test) {
        return Err(CheckError::from_message(format!(
            "mirror test {:?} is not Cargo-test reachable test source",
            mirror.test
        )));
    }
    crate::mirror_execution::validate(
        mirror,
        &model.cargo,
        &model.feature_worlds,
        &model.compilation_domains,
        test,
    )
    .map_err(CheckError::from_message)?;
    if !production.packages.contains(&mirror.execution.package)
        || !test.packages.contains(&mirror.execution.package)
    {
        return Err(CheckError::from_message(format!(
            "mirror execution package {:?} does not own both exact sources",
            mirror.execution.package
        )));
    }
    Ok(())
}

fn exact_file<'a>(
    model: &'a RepositoryModel,
    path: &str,
    label: &str,
) -> Result<&'a RustFileFacts, CheckError> {
    let matches = model
        .source
        .files
        .iter()
        .filter(|file| file.relative == path)
        .collect::<Vec<_>>();
    let [file] = matches.as_slice() else {
        return Err(CheckError::from_message(format!(
            "mirror {label} {path:?} does not select exactly one analyzed Rust source"
        )));
    };
    Ok(file)
}

#[cfg(test)]
#[path = "mirrors_test.rs"]
mod mirrors_test;
