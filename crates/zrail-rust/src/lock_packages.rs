//! Bounded whole-lock inventories reuse complete offline Cargo resolution without execution.

mod identity;
mod model;

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::{AnalysisQuality, Finding, FindingSink, LockPackageAssertion, LockPackageRule};

use crate::cargo::ResolvedCargoGraph;
use identity::Identity;
pub use model::{GovernedLockPackage, LockPackageDifference};

const MAX_COMPARISON_WORK: usize = 64 * 1_024 * 1_024;

pub(crate) fn analyze(
    graph: Option<&ResolvedCargoGraph>,
    rules: &[LockPackageRule],
) -> Result<Vec<GovernedLockPackage>, String> {
    if rules.is_empty() {
        return Ok(Vec::new());
    }
    let graph =
        graph.ok_or_else(|| "whole-lock package inventories require Cargo.lock".to_owned())?;
    let mut by_name = BTreeMap::<&str, Vec<Identity<'_>>>::new();
    let mut lock_node_count = 0;
    for node in graph.all_packages() {
        by_name
            .entry(&node.name)
            .or_default()
            .push(Identity::from(node));
        lock_node_count += 1;
    }
    let mut rules = rules.iter().collect::<Vec<_>>();
    rules.sort_by(|left, right| left.name.cmp(&right.name));
    let mut reports = Vec::new();
    let mut work = 0_usize;
    for rule in rules {
        let selected = by_name
            .get(rule.package.as_str())
            .map_or(&[][..], Vec::as_slice);
        for identity in selected {
            work = work.saturating_add(identity.work());
            if work > MAX_COMPARISON_WORK {
                return Err(format!(
                    "whole-lock inventories exceed the {MAX_COMPARISON_WORK}-unit comparison safety limit at {:?}",
                    rule.name
                ));
            }
        }
        let mut policy = rule.clone();
        let exact_difference = match &mut policy.assertion {
            LockPackageAssertion::Count { .. } => None,
            LockPackageAssertion::Exact { identities } => {
                identities.sort();
                let expected = identities
                    .iter()
                    .map(Identity::from)
                    .collect::<BTreeSet<_>>();
                let observed = selected.iter().copied().collect::<BTreeSet<_>>();
                let missing = expected
                    .difference(&observed)
                    .map(|identity| identity.owned())
                    .collect();
                let unexpected_count = observed.difference(&expected).count();
                let unexpected_sample = identity::sample(observed.difference(&expected).copied());
                Some(LockPackageDifference {
                    missing,
                    unexpected_count,
                    unexpected_omitted: unexpected_count - unexpected_sample.len(),
                    unexpected_sample,
                })
            }
        };
        let satisfied = match &policy.assertion {
            LockPackageAssertion::Count { count } => *count == selected.len(),
            LockPackageAssertion::Exact { .. } => {
                exact_difference.as_ref().is_some_and(|difference| {
                    difference.missing.is_empty() && difference.unexpected_count == 0
                })
            }
        };
        let observed_sample = identity::sample(selected.iter().copied());
        reports.push(GovernedLockPackage {
            policy_id: format!("dependency:lock-package:{}", rule.name),
            policy,
            scope: "whole-cargo-lock".into(),
            quality: AnalysisQuality::Exact,
            lock_sha256: graph.lock_sha256().into(),
            lock_node_count,
            observed_count: selected.len(),
            observed_omitted: selected.len() - observed_sample.len(),
            observed_sample,
            exact_difference,
            satisfied,
        });
    }
    Ok(reports)
}

pub(crate) fn evaluate(policies: &[GovernedLockPackage], findings: &mut FindingSink) {
    for policy in policies.iter().filter(|policy| !policy.satisfied) {
        let message = match &policy.policy.assertion {
            LockPackageAssertion::Count { count } => format!(
                "Cargo.lock contains {} nodes named {:?}; expected exactly {count}",
                policy.observed_count, policy.policy.package,
            ),
            LockPackageAssertion::Exact { identities } => format!(
                "Cargo.lock identity set for {:?} differs: {} observed, {} expected, {} missing, {} unexpected",
                policy.policy.package,
                policy.observed_count,
                identities.len(),
                policy
                    .exact_difference
                    .as_ref()
                    .map_or(0, |difference| difference.missing.len()),
                policy
                    .exact_difference
                    .as_ref()
                    .map_or(0, |difference| difference.unexpected_count),
            ),
        };
        findings.push(Finding::error(
            "DEP-LOCK-001", &policy.policy_id, "dependency", message,
        ).at("Cargo.lock", None).because(&policy.policy.reason)
            .with_help("restore the reviewed whole-lock inventory; use coverage or explain Cargo.lock for exact identity differences"));
    }
}

pub(crate) fn display(policies: &[GovernedLockPackage]) -> String {
    if policies.is_empty() {
        return "<none>".into();
    }
    policies.iter().map(|policy| format!(
        "{}: package {:?}; {:?}; {} of {} whole-lock nodes; satisfied {}; sha256 {}; reason {}",
        policy.policy_id, policy.policy.package, policy.policy.assertion, policy.observed_count,
        policy.lock_node_count, policy.satisfied, policy.lock_sha256, policy.policy.reason,
    )).collect::<Vec<_>>().join("; ")
}

pub(crate) fn for_path(policies: &[GovernedLockPackage], path: &str) -> Vec<GovernedLockPackage> {
    if path == "Cargo.lock" {
        policies.to_vec()
    } else {
        Vec::new()
    }
}

#[cfg(test)]
#[path = "lock_packages_test.rs"]
mod lock_packages_test;
