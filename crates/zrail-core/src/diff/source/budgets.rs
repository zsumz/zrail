//! Scoped size permissions compare thresholds, selectors, and exact exceptions separately.

mod baselines;
mod exceptions;

use std::collections::{BTreeMap, BTreeSet};

use crate::{Contract, ScopedBudgetContract, SizePolicyContract, SizeTargetMode, SizeThresholds};

use super::super::{ArchitectureChange, ChangeKind, support::compare_number};

pub(super) fn compare(before: &Contract, after: &Contract, changes: &mut Vec<ArchitectureChange>) {
    let left = &before.source.rust.budgets;
    let right = &after.source.rust.budgets;
    if left.is_some() != right.is_some() {
        changes.push(ArchitectureChange::new(
            if right.is_some() {
                ChangeKind::Revoke
            } else {
                ChangeKind::Grant
            },
            "rust.file-size",
            "source.rust.budgets",
            "independent hard-ceiling enforcement changed",
        ));
    }
    let empty = SizePolicyContract::default();
    let left = left.as_ref().unwrap_or(&empty);
    let right = right.as_ref().unwrap_or(&empty);
    exceptions::metadata(left.exception_metadata, right.exception_metadata, changes);
    let old = by_name(&left.overrides);
    let new = by_name(&right.overrides);
    for name in old
        .keys()
        .chain(new.keys())
        .copied()
        .collect::<BTreeSet<_>>()
    {
        let subject = format!("rust:size:scope:{name}");
        match (old.get(name), new.get(name)) {
            (Some(left), Some(right)) => {
                if !same_selection(left, right) {
                    changes.push(ArchitectureChange::new(
                        ChangeKind::Unknown,
                        "rust.file-size",
                        &subject,
                        "budget selector changed; path/package/role intersections require review",
                    ));
                }
                thresholds(&subject, left.budget, right.budget, changes);
                if left.reason != right.reason {
                    changes.push(
                        ArchitectureChange::new(
                            ChangeKind::Unknown,
                            "rust.file-size",
                            &subject,
                            "budget justification changed",
                        )
                        .values(&left.reason, &right.reason),
                    );
                }
            }
            (None, Some(scope)) => fallback(before, scope, false, &subject, changes),
            (Some(scope), None) => fallback(after, scope, true, &subject, changes),
            (None, None) => {}
        }
    }
    exceptions::compare(&left.exceptions, &right.exceptions, changes);
    baselines::compare(&before.ratchets, &after.ratchets, changes);
}

fn by_name(scopes: &[ScopedBudgetContract]) -> BTreeMap<&str, &ScopedBudgetContract> {
    scopes
        .iter()
        .map(|scope| (scope.name.as_str(), scope))
        .collect()
}

fn same_selection(left: &ScopedBudgetContract, right: &ScopedBudgetContract) -> bool {
    same_set(&left.include, &right.include)
        && same_set(&left.exclude, &right.exclude)
        && same_set(&left.packages, &right.packages)
        && same_set(&left.roles, &right.roles)
}

fn same_set<T: Ord>(left: &[T], right: &[T]) -> bool {
    left.iter().collect::<BTreeSet<_>>() == right.iter().collect::<BTreeSet<_>>()
}

fn thresholds(
    subject: &str,
    left: SizeThresholds,
    right: SizeThresholds,
    changes: &mut Vec<ArchitectureChange>,
) {
    for (name, old, new) in [
        ("target", left.target, right.target),
        ("hard", left.hard, right.hard),
    ] {
        compare_number(
            "rust.file-size",
            &format!("{subject}.{name}"),
            old,
            new,
            changes,
        );
    }
    match (left.soft, right.soft) {
        (Some(left), Some(right)) => compare_number(
            "rust.file-size",
            &format!("{subject}.soft"),
            left,
            right,
            changes,
        ),
        (None, None) => {}
        (_, new) => changes.push(ArchitectureChange::new(
            if new.is_some() {
                ChangeKind::Revoke
            } else {
                ChangeKind::Grant
            },
            "rust.file-size",
            format!("{subject}.soft"),
            "soft-threshold warning coverage changed",
        )),
    }
    if left.target_mode != right.target_mode {
        changes.push(ArchitectureChange::new(
            if right.target_mode == SizeTargetMode::Warn {
                ChangeKind::Grant
            } else {
                ChangeKind::Revoke
            },
            "rust.file-size",
            format!("{subject}.target-mode"),
            "design-target enforcement changed",
        ));
    }
}

fn fallback(
    contract: &Contract,
    scope: &ScopedBudgetContract,
    removed: bool,
    subject: &str,
    changes: &mut Vec<ArchitectureChange>,
) {
    // A selector can overlap a differently named override or generated authority.
    // Do not guess at glob inclusion or synthesize a default that is not known.
    if contract
        .source
        .rust
        .budgets
        .as_ref()
        .is_some_and(|policy| !policy.overrides.is_empty())
        || !contract.source.rust.generated.is_empty()
    {
        changes.push(ArchitectureChange::new(ChangeKind::Unknown, "rust.file-size", subject,
            "added or removed budget requires comparison with other effective scoped/generated authority"));
        return;
    }
    let Some(size) = &contract.source.rust.size else {
        changes.push(ArchitectureChange::new(
            if removed {
                ChangeKind::Grant
            } else {
                ChangeKind::Revoke
            },
            "rust.file-size",
            subject,
            "scoped file-size enforcement changed",
        ));
        return;
    };
    // Compare every possible default. Conflicting directions remain visible;
    // protected review sees any grant instead of relying on path guesses.
    for (role, budget) in [
        ("facade", size.facade),
        ("implementation", size.implementation),
        ("test", size.test),
        ("auxiliary", size.auxiliary),
    ] {
        let default = SizeThresholds {
            target: budget.target,
            soft: None,
            hard: budget.hard,
            target_mode: SizeTargetMode::Error,
        };
        let (left, right) = if removed {
            (scope.budget, default)
        } else {
            (default, scope.budget)
        };
        thresholds(&format!("{subject}.default-{role}"), left, right, changes);
    }
}

#[cfg(test)]
#[path = "budgets_test.rs"]
mod budgets_test;
