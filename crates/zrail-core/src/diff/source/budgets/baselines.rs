//! Authored measurements cannot weaken silently behind an unchanged lock value.

use std::collections::{BTreeMap, BTreeSet};

use crate::RatchetContract;

use super::super::super::{ArchitectureChange, ChangeKind, support::compare_number};

pub(super) fn compare(
    before: &[RatchetContract],
    after: &[RatchetContract],
    changes: &mut Vec<ArchitectureChange>,
) {
    let old = by_path(before);
    let new = by_path(after);
    for path in old
        .keys()
        .chain(new.keys())
        .copied()
        .collect::<BTreeSet<_>>()
    {
        let left = old.get(path).and_then(|ratchet| ratchet.baseline);
        let right = new.get(path).and_then(|ratchet| ratchet.baseline);
        let subject = format!("ratchet:rust.file-size:{path}.baseline");
        match (left, right) {
            (None, None) => {}
            (Some(left), Some(right)) => {
                compare_number("rust.file-size.baseline", &subject, left, right, changes);
            }
            (_, new) => changes.push(ArchitectureChange::new(
                if new.is_some() {
                    ChangeKind::Revoke
                } else {
                    ChangeKind::Grant
                },
                "rust.file-size.baseline",
                subject,
                "exact authored baseline requirement changed",
            )),
        }
    }
}

fn by_path(ratchets: &[RatchetContract]) -> BTreeMap<&str, &RatchetContract> {
    ratchets
        .iter()
        .filter(|ratchet| ratchet.rule == "rust.file-size")
        .map(|ratchet| (ratchet.target.as_str(), ratchet))
        .collect()
}
