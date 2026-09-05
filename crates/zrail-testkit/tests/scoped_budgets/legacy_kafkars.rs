//! Frozen Kafkars predicates execute only in trusted differential qualification.

use std::collections::BTreeMap;

use super::Budget;

pub(super) fn violations(lines: usize, baseline: Option<usize>, allowed: bool) -> Vec<String> {
    let baseline = baseline.map(|lines| super::BudgetBaseline {
        lines,
        reason: "Reviewed measurement.".into(),
    });
    let allowance = super::BudgetAllow {
        reason: "Reviewed allowance.".into(),
        owner: "architecture".into(),
        issue: "ARCH-42".into(),
    };
    let baselines = baseline
        .as_ref()
        .map(|entry| ("src/child.rs", entry))
        .into_iter()
        .collect();
    let allows = allowed
        .then_some(("src/child.rs", &allowance))
        .into_iter()
        .collect();
    let budget = Budget {
        target: 5,
        soft: 8,
        hard: 10,
    };
    let mut violations = Vec::new();
    check_baseline("src/child.rs", lines, budget, &baselines, &mut violations);
    check_allow("src/child.rs", lines, budget, &allows, &mut violations);
    violations
}

pub(super) fn invalid_metadata(field: super::MetadataField) -> Vec<String> {
    let mut baseline = super::BudgetBaseline {
        lines: 11,
        reason: "Reviewed measurement.".into(),
    };
    let mut allowance = super::BudgetAllow {
        reason: "Reviewed allowance.".into(),
        owner: "architecture".into(),
        issue: "ARCH-42".into(),
    };
    match field {
        super::MetadataField::BaselineReason => baseline.reason.clear(),
        super::MetadataField::HardReason => allowance.reason.clear(),
        super::MetadataField::Owner => allowance.owner.clear(),
        super::MetadataField::Issue => allowance.issue.clear(),
    }
    let budget = Budget {
        target: 5,
        soft: 8,
        hard: 10,
    };
    let mut violations = Vec::new();
    check_baseline(
        "src/child.rs",
        11,
        budget,
        &BTreeMap::from([("src/child.rs", &baseline)]),
        &mut violations,
    );
    check_allow(
        "src/child.rs",
        11,
        budget,
        &BTreeMap::from([("src/child.rs", &allowance)]),
        &mut violations,
    );
    violations
}

fn check_baseline(
    relative: &str,
    lines: usize,
    budget: Budget,
    baselines: &BTreeMap<&str, &super::BudgetBaseline>,
    violations: &mut Vec<String>,
) {
    if lines > budget.target {
        match baselines.get(relative) {
            Some(entry) if entry.reason.trim().is_empty() => {
                violations.push(format!("{relative} has an unexplained baseline"));
            }
            Some(entry) if lines != entry.lines => {
                let direction = if lines > entry.lines {
                    "grew beyond"
                } else {
                    "shrunk below"
                };
                violations.push(format!(
                    "{relative} {direction} its exact {}-line baseline to {lines} lines",
                    entry.lines
                ));
            }
            Some(_) => {}
            None => violations.push(format!(
                "{relative} is {lines} lines, above its {}-line design target{}",
                budget.target,
                if lines > budget.soft {
                    " and soft limit"
                } else {
                    ""
                }
            )),
        }
    } else if baselines.contains_key(relative) {
        violations.push(format!("{relative} has a stale baseline"));
    }
}

fn check_allow(
    relative: &str,
    lines: usize,
    budget: Budget,
    allows: &BTreeMap<&str, &super::BudgetAllow>,
    violations: &mut Vec<String>,
) {
    if lines > budget.hard {
        match allows.get(relative) {
            Some(entry)
                if !entry.reason.trim().is_empty()
                    && !entry.owner.trim().is_empty()
                    && !entry.issue.trim().is_empty() => {}
            _ => violations.push(format!(
                "{relative} exceeds its {}-line hard ceiling without a reviewed allow",
                budget.hard
            )),
        }
    } else if allows.contains_key(relative) {
        violations.push(format!("{relative} has a stale hard-ceiling allow"));
    }
}
