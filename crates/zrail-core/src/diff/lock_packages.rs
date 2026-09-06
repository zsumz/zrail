//! Whole-lock inventories compare accepted identities and quantities, never numeric direction.

use std::collections::{BTreeMap, BTreeSet};

use crate::{Contract, LockPackageAssertion, LockPackageRule};

use super::{ArchitectureChange, ChangeKind};

pub(super) fn compare(before: &Contract, after: &Contract, changes: &mut Vec<ArchitectureChange>) {
    let old = named(&before.dependencies.lock_packages);
    let new = named(&after.dependencies.lock_packages);
    for name in old
        .keys()
        .chain(new.keys())
        .copied()
        .collect::<BTreeSet<_>>()
    {
        match (old.get(name), new.get(name)) {
            (Some(left), Some(right)) => {
                if left.package == right.package {
                    if !implies(&right.assertion, &left.assertion) {
                        changed(
                            changes,
                            name,
                            "assertion",
                            ChangeKind::Grant,
                            &left.assertion,
                            &right.assertion,
                        );
                    }
                    if !implies(&left.assertion, &right.assertion) {
                        changed(
                            changes,
                            name,
                            "assertion",
                            ChangeKind::Revoke,
                            &left.assertion,
                            &right.assertion,
                        );
                    }
                } else {
                    changed(
                        changes,
                        name,
                        "package selection",
                        ChangeKind::Grant,
                        left,
                        right,
                    );
                    changed(
                        changes,
                        name,
                        "package selection",
                        ChangeKind::Revoke,
                        left,
                        right,
                    );
                }
                if left.reason != right.reason {
                    changed(
                        changes,
                        name,
                        "justification",
                        ChangeKind::Unknown,
                        &left.reason,
                        &right.reason,
                    );
                }
            }
            (Some(rule), None) => changed(
                changes,
                name,
                "required guard",
                ChangeKind::Grant,
                rule,
                &"removed",
            ),
            (None, Some(rule)) => changed(
                changes,
                name,
                "required guard",
                ChangeKind::Revoke,
                &"absent",
                rule,
            ),
            (None, None) => {}
        }
    }
}

fn implies(left: &LockPackageAssertion, right: &LockPackageAssertion) -> bool {
    use LockPackageAssertion::{Count, Exact};
    match (left, right) {
        (Count { count: left }, Count { count: right }) => left == right,
        (Exact { identities }, Count { count }) => identities.len() == *count,
        (Count { count }, Exact { identities }) => *count == 0 && identities.is_empty(),
        (Exact { identities: left }, Exact { identities: right }) => {
            left.iter().collect::<BTreeSet<_>>() == right.iter().collect::<BTreeSet<_>>()
        }
    }
}

fn named(rules: &[LockPackageRule]) -> BTreeMap<&str, &LockPackageRule> {
    rules
        .iter()
        .map(|rule| (rule.name.as_str(), rule))
        .collect()
}

fn changed(
    changes: &mut Vec<ArchitectureChange>,
    name: &str,
    field: &str,
    kind: ChangeKind,
    before: &impl std::fmt::Debug,
    after: &impl std::fmt::Debug,
) {
    changes.push(
        ArchitectureChange::new(
            kind,
            "dependencies.lock-package",
            format!("dependency:lock-package:{name}"),
            format!(
                "whole-Cargo.lock {field} changed; identities include version, source, and checksum"
            ),
        )
        .values(format!("{before:?}"), format!("{after:?}")),
    );
}

#[cfg(test)]
#[path = "lock_packages_test.rs"]
mod lock_packages_test;
