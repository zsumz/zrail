//! Semantic comparison of content-addressed qualification gates.

use std::collections::{BTreeMap, BTreeSet};

use crate::{LockFile, LockedGate, LockedGateInput};

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
            (Some(left), Some(right)) => compare_gate(left, right, changes),
            _ => {}
        }
    }
}

fn compare_gate(before: &LockedGate, after: &LockedGate, changes: &mut Vec<ArchitectureChange>) {
    if before.path != after.path || before.sha256 != after.sha256 {
        changes.push(
            ArchitectureChange::new(
                ChangeKind::Unknown,
                "qualification.gate-lock",
                &before.name,
                "reviewed qualification gate contents changed",
            )
            .values(value(before), value(after)),
        );
    }
    compare_inputs(before, after, changes);
}

fn compare_inputs(before: &LockedGate, after: &LockedGate, changes: &mut Vec<ArchitectureChange>) {
    let old = inputs_by_path(&before.inputs);
    let new = inputs_by_path(&after.inputs);
    for path in old
        .keys()
        .chain(new.keys())
        .copied()
        .collect::<BTreeSet<_>>()
    {
        let target = format!("{}:{path}", before.name);
        match (old.get(path), new.get(path)) {
            (None, Some(_)) => changes.push(ArchitectureChange::new(
                ChangeKind::Revoke,
                "qualification.gate-input-lock",
                target,
                "qualification input became content-addressed",
            )),
            (Some(_), None) => changes.push(ArchitectureChange::new(
                ChangeKind::Grant,
                "qualification.gate-input-lock",
                target,
                "qualification input is no longer content-addressed",
            )),
            (Some(left), Some(right)) if left.sha256 != right.sha256 => changes.push(
                ArchitectureChange::new(
                    ChangeKind::Unknown,
                    "qualification.gate-input-lock",
                    target,
                    "reviewed qualification input contents changed",
                )
                .values(&left.sha256, &right.sha256),
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

fn inputs_by_path(inputs: &[LockedGateInput]) -> BTreeMap<&str, &LockedGateInput> {
    inputs
        .iter()
        .map(|input| (input.path.as_str(), input))
        .collect()
}
