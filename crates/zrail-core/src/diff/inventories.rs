//! Inventory review compares accepted quantity states, not numeric direction alone.

use std::collections::{BTreeMap, BTreeSet};

use crate::{Contract, RustInventoryAssertion, RustInventoryRule};

use super::{ArchitectureChange, ChangeKind};

pub(super) fn compare(before: &Contract, after: &Contract, changes: &mut Vec<ArchitectureChange>) {
    let old = named(&before.source.rust.inventories);
    let new = named(&after.source.rust.inventories);
    for name in old
        .keys()
        .chain(new.keys())
        .copied()
        .collect::<BTreeSet<_>>()
    {
        match (old.get(name), new.get(name)) {
            (Some(left), Some(right)) => {
                let selection = selection_change(left, right);
                if let Some(kind) = selection {
                    changed(changes, name, "selection/world", kind, left, right);
                }
                if !implies(&right.assertion, &left.assertion) {
                    changed(
                        changes,
                        name,
                        "quantity",
                        ChangeKind::Grant,
                        &left.assertion,
                        &right.assertion,
                    );
                }
                if !implies(&left.assertion, &right.assertion) {
                    changed(
                        changes,
                        name,
                        "quantity",
                        ChangeKind::Revoke,
                        &left.assertion,
                        &right.assertion,
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

fn selection_change(left: &RustInventoryRule, right: &RustInventoryRule) -> Option<ChangeKind> {
    if std::mem::discriminant(&left.subject) != std::mem::discriminant(&right.subject) {
        return Some(ChangeKind::Unknown);
    }
    let (li, le) = (set(&left.include), set(&left.exclude));
    let (ri, re) = (set(&right.include), set(&right.exclude));
    let subject_expanded = subject_subset(&left.subject, &right.subject);
    let subject_narrowed = subject_subset(&right.subject, &left.subject);
    if li == ri && le == re && left.world == right.world && subject_expanded && subject_narrowed {
        return None;
    }
    if left.world != right.world
        || !implies(&left.assertion, &right.assertion)
        || !implies(&right.assertion, &left.assertion)
    {
        return Some(ChangeKind::Unknown);
    }
    let expanded = li.is_subset(&ri) && re.is_subset(&le) && subject_expanded;
    let narrowed = ri.is_subset(&li) && le.is_subset(&re) && subject_narrowed;
    if !expanded && !narrowed {
        return Some(ChangeKind::Unknown);
    }
    let expansion_tightens = match &left.assertion {
        RustInventoryAssertion::Count { minimum: 0, .. }
        | RustInventoryAssertion::ExactCounts { .. }
        | RustInventoryAssertion::ExactOwners { .. } => true,
        RustInventoryAssertion::Count { maximum: None, .. } => false,
        RustInventoryAssertion::Count { .. } => return Some(ChangeKind::Unknown),
    };
    Some(if expanded == expansion_tightens {
        ChangeKind::Revoke
    } else {
        ChangeKind::Grant
    })
}

fn subject_subset(left: &crate::RustInventorySubject, right: &crate::RustInventorySubject) -> bool {
    if !set(left.names()).is_subset(&set(right.names())) {
        return false;
    }
    if let (
        crate::RustInventorySubject::WrittenTraitImpls {
            implementing_types: old,
            ..
        },
        crate::RustInventorySubject::WrittenTraitImpls {
            implementing_types: new,
            ..
        },
    ) = (left, right)
    {
        left.names()
            .iter()
            .all(|name| match (old.get(name), new.get(name)) {
                (_, None) => true,
                (None, Some(_)) => false,
                (Some(before), Some(after)) => set(before).is_subset(&set(after)),
            })
    } else {
        true
    }
}

fn implies(left: &RustInventoryAssertion, right: &RustInventoryAssertion) -> bool {
    use RustInventoryAssertion::{Count, ExactCounts, ExactOwners};
    match (left, right) {
        (
            Count {
                minimum: lm,
                maximum: lx,
            },
            Count {
                minimum: rm,
                maximum: rx,
            },
        ) => {
            lm >= rm && rx.is_none_or(|right_max| lx.is_some_and(|left_max| left_max <= right_max))
        }
        (ExactCounts { counts }, Count { minimum, maximum }) => {
            let total = counts.iter().map(|count| count.count).sum::<usize>();
            total >= *minimum && maximum.is_none_or(|maximum| total <= maximum)
        }
        (
            Count {
                minimum: 0,
                maximum: Some(0),
            },
            ExactCounts { counts },
        ) => counts.is_empty(),
        (ExactCounts { counts: left }, ExactCounts { counts: right }) => {
            left.iter().collect::<BTreeSet<_>>() == right.iter().collect::<BTreeSet<_>>()
        }
        (ExactOwners { owners: left }, ExactOwners { owners: right }) => {
            left.iter().collect::<BTreeSet<_>>() == right.iter().collect::<BTreeSet<_>>()
        }
        (ExactCounts { counts }, ExactOwners { owners }) => {
            counts
                .iter()
                .map(|count| (&count.path, &count.name))
                .collect::<BTreeSet<_>>()
                == owners
                    .iter()
                    .map(|owner| (&owner.path, &owner.name))
                    .collect::<BTreeSet<_>>()
        }
        (ExactOwners { owners }, ExactCounts { counts }) => owners.is_empty() && counts.is_empty(),
        (ExactOwners { owners }, Count { minimum, maximum }) => {
            owners.len() >= *minimum && (maximum.is_none() || owners.is_empty())
        }
        (
            Count {
                minimum: 0,
                maximum: Some(0),
            },
            ExactOwners { owners },
        ) => owners.is_empty(),
        (Count { .. }, ExactCounts { .. } | ExactOwners { .. }) => false,
    }
}

fn set(values: &[String]) -> BTreeSet<&str> {
    values.iter().map(String::as_str).collect()
}

fn named(rules: &[RustInventoryRule]) -> BTreeMap<&str, &RustInventoryRule> {
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
    changes.push(ArchitectureChange::new(
        kind, "source.rust.inventories", format!("rust:inventory:{name}"),
        format!("Rust inventory {field} changed; exact quantities retain location and subject identity"),
    ).values(format!("{before:?}"), format!("{after:?}")));
}

#[cfg(test)]
#[path = "inventories_test.rs"]
mod inventories_test;
