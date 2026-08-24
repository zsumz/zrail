//! Reviewed analysis-budget overrides remain visible in semantic review.

use crate::Contract;

use super::{ArchitectureChange, ChangeKind};

pub(super) fn compare(before: &Contract, after: &Contract, changes: &mut Vec<ArchitectureChange>) {
    let before = before.analysis.limits;
    let after = after.analysis.limits;
    compare_limit(
        "derived-source-instances",
        before.derived_source_instances,
        after.derived_source_instances,
        changes,
    );
    compare_limit(
        "include-projection-work",
        before.include_projection_work,
        after.include_projection_work,
        changes,
    );
    compare_limit(
        "projected-facts",
        before.projected_facts,
        after.projected_facts,
        changes,
    );
}

fn compare_limit(
    subject: &str,
    before: Option<usize>,
    after: Option<usize>,
    changes: &mut Vec<ArchitectureChange>,
) {
    if before == after {
        return;
    }
    let kind = match (before, after) {
        (None, Some(_)) => ChangeKind::Grant,
        (Some(_), None) => ChangeKind::Revoke,
        (Some(left), Some(right)) if right > left => ChangeKind::Grant,
        _ => ChangeKind::Revoke,
    };
    changes.push(
        ArchitectureChange::new(
            kind,
            "analysis.limit",
            subject,
            "changes a content-bound multiplicative analysis budget",
        )
        .values(display(before), display(after)),
    );
}

fn display(value: Option<usize>) -> String {
    value.map_or_else(|| "input-derived".into(), |value| value.to_string())
}

#[cfg(test)]
#[path = "analysis_test.rs"]
mod analysis_test;
