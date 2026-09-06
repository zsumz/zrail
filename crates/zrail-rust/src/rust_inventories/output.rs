//! Quantities and physical locations remain distinct in diagnostics and explanation.

use zrail_core::{
    Finding, FindingSink, RustInventoryAssertion, RustInventorySubject, glob_matches,
};

use super::GovernedRustInventory;

pub(crate) fn evaluate(policies: &[GovernedRustInventory], findings: &mut FindingSink) {
    for policy in policies.iter().filter(|policy| !policy.satisfied) {
        let subject = match policy.policy.subject {
            RustInventorySubject::WrittenMethods { .. } => "authored method-call",
            RustInventorySubject::WrittenExpressionPaths { .. } => "authored expression-path",
        };
        let expected = match &policy.policy.assertion {
            RustInventoryAssertion::Count { minimum, maximum } => {
                format!(
                    "{minimum}..{} occurrences",
                    maximum.map_or_else(|| "unbounded".into(), |max| max.to_string())
                )
            }
            RustInventoryAssertion::ExactCounts { counts } => {
                format!("the exact {} file/subject quantities", counts.len())
            }
            RustInventoryAssertion::ExactOwners { owners } => {
                format!(
                    "the exact {} file/subject owners regardless of quantity",
                    owners.len()
                )
            }
        };
        findings.push(Finding::error(
            "RUST-INVENTORY-001", &policy.policy_id, "source",
            format!("{} observed {} {subject} occurrences in {} physical files; required {expected}",
                policy.policy_id, policy.observed_count, policy.inputs.len()),
        ).because(&policy.policy.reason).with_help(
            "restore the reviewed written syntax inventory; coverage reports every file/subject count and bounded source locations",
        ));
    }
}

pub(crate) fn for_path(
    policies: &[GovernedRustInventory],
    path: &str,
) -> Vec<GovernedRustInventory> {
    policies
        .iter()
        .filter(|report| {
            report
                .policy
                .include
                .iter()
                .any(|pattern| glob_matches(pattern, path))
                && !report
                    .policy
                    .exclude
                    .iter()
                    .any(|pattern| glob_matches(pattern, path))
        })
        .cloned()
        .collect()
}

pub(crate) fn display(policies: &[GovernedRustInventory]) -> String {
    if policies.is_empty() {
        return "<none>".into();
    }
    policies.iter().map(|policy| format!(
        "{}: {}; {:?}; {:?}; include {:?}, exclude {:?}; {:?}; {} occurrences in {} files; counts {:?}; satisfied {}; reason {}",
        policy.policy_id, policy.claim, policy.policy.world, policy.policy.subject,
        policy.policy.include, policy.policy.exclude, policy.policy.assertion,
        policy.observed_count, policy.inputs.len(), policy.counts, policy.satisfied, policy.policy.reason,
    )).collect::<Vec<_>>().join("; ")
}
