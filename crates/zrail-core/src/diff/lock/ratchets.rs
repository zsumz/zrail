//! Directional comparison of selector-aware tightening ratchets.

use std::collections::{BTreeMap, BTreeSet};

use crate::{LockFile, LockedRatchet};

use super::super::{ArchitectureChange, ChangeKind};

type RatchetIdentity = (String, Option<String>, String);

pub(super) fn compare(before: &LockFile, after: &LockFile, changes: &mut Vec<ArchitectureChange>) {
    let old = by_identity(&before.ratchets);
    let new = by_identity(&after.ratchets);
    let identities = old
        .keys()
        .chain(new.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    for identity in identities {
        let subject = label(&identity);
        match (old.get(&identity), new.get(&identity)) {
            (Some(left), Some(right)) if left.value < right.value => changes.push(
                ArchitectureChange::new(
                    ChangeKind::Debt,
                    "ratchet",
                    &subject,
                    "ratchet ceiling increased",
                )
                .values(left.value.to_string(), right.value.to_string()),
            ),
            (Some(left), Some(right)) if left.value > right.value => changes.push(
                ArchitectureChange::new(
                    ChangeKind::Cleanup,
                    "ratchet",
                    &subject,
                    "ratchet tightened with the repository",
                )
                .values(left.value.to_string(), right.value.to_string()),
            ),
            (None, Some(right)) => changes.push(
                ArchitectureChange::new(
                    ChangeKind::Debt,
                    "ratchet",
                    &subject,
                    "reviewed architecture debt was recorded",
                )
                .values("<none>", right.value.to_string()),
            ),
            (Some(left), None) => changes.push(
                ArchitectureChange::new(
                    ChangeKind::Cleanup,
                    "ratchet",
                    &subject,
                    "reviewed architecture debt was removed",
                )
                .values(left.value.to_string(), "<none>"),
            ),
            _ => {}
        }
    }
}

fn by_identity(ratchets: &[LockedRatchet]) -> BTreeMap<RatchetIdentity, &LockedRatchet> {
    ratchets
        .iter()
        .map(|ratchet| {
            (
                (
                    ratchet.rule.clone(),
                    ratchet
                        .selector
                        .as_deref()
                        .map(crate::normalize_ratchet_selector),
                    ratchet.target.clone(),
                ),
                ratchet,
            )
        })
        .collect()
}

fn label((rule, selector, target): &RatchetIdentity) -> String {
    selector.as_ref().map_or_else(
        || format!("{rule}:{target}"),
        |selector| format!("{rule}[{selector}]:{target}"),
    )
}
