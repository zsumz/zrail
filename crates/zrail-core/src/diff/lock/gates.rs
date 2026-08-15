//! Semantic comparison of content-addressed qualification gates.

use std::collections::{BTreeMap, BTreeSet};

use crate::{LockFile, LockedGate};

use super::super::{ArchitectureChange, ChangeKind};

pub(super) fn compare(before: &LockFile, after: &LockFile, changes: &mut Vec<ArchitectureChange>) {
    let old = by_name(&before.gates);
    let new = by_name(&after.gates);
    let names = old
        .keys()
        .chain(new.keys())
        .copied()
        .collect::<BTreeSet<_>>();
    for name in names {
        match (old.get(name), new.get(name)) {
            (None, Some(_)) => changes.push(ArchitectureChange::new(
                ChangeKind::Revoke,
                "qualification.gate-lock",
                name,
                "qualification gate became content-addressed",
            )),
            (Some(_), None) => changes.push(ArchitectureChange::new(
                ChangeKind::Grant,
                "qualification.gate-lock",
                name,
                "qualification gate is no longer content-addressed",
            )),
            (Some(left), Some(right)) if left != right => changes.push(
                ArchitectureChange::new(
                    ChangeKind::Unknown,
                    "qualification.gate-lock",
                    name,
                    "reviewed qualification gate contents changed",
                )
                .values(value(left), value(right)),
            ),
            _ => {}
        }
    }
}

fn by_name(gates: &[LockedGate]) -> BTreeMap<&str, &LockedGate> {
    gates
        .iter()
        .map(|gate| (gate.name.as_str(), gate))
        .collect()
}

fn value(gate: &LockedGate) -> String {
    format!("{}@{}", gate.path, gate.sha256)
}
