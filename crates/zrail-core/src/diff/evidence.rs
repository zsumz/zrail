//! Semantic changes to invariant evidence and qualification gates.

use std::collections::{BTreeMap, BTreeSet};

use crate::{Contract, GateContract, InvariantContract};

use super::{ArchitectureChange, ChangeKind, support::compare_named_set};

pub(super) fn compare(before: &Contract, after: &Contract, changes: &mut Vec<ArchitectureChange>) {
    compare_gates(&before.gates, &after.gates, changes);
    compare_invariants(&before.invariants, &after.invariants, changes);
}

fn compare_gates(
    before: &[GateContract],
    after: &[GateContract],
    changes: &mut Vec<ArchitectureChange>,
) {
    let old = gates_by_name(before);
    let new = gates_by_name(after);
    for name in old
        .keys()
        .chain(new.keys())
        .copied()
        .collect::<BTreeSet<_>>()
    {
        match (old.get(name), new.get(name)) {
            (None, Some(_)) => changes.push(ArchitectureChange::new(
                ChangeKind::Revoke,
                "qualification.gate",
                name,
                "new qualification gate was declared",
            )),
            (Some(_), None) => changes.push(ArchitectureChange::new(
                ChangeKind::Grant,
                "qualification.gate",
                name,
                "qualification gate was removed",
            )),
            (Some(left), Some(right)) => compare_gate(left, right, changes),
            (None, None) => {}
        }
    }
}

fn compare_gate(
    before: &GateContract,
    after: &GateContract,
    changes: &mut Vec<ArchitectureChange>,
) {
    if before.kind != after.kind {
        changes.push(
            ArchitectureChange::new(
                ChangeKind::Unknown,
                "qualification.gate-kind",
                &before.name,
                "qualification gate kind changed and requires review",
            )
            .values(format!("{:?}", before.kind), format!("{:?}", after.kind)),
        );
    }
    if before.path != after.path {
        changes.push(
            ArchitectureChange::new(
                ChangeKind::Unknown,
                "qualification.gate-path",
                &before.name,
                "qualification gate file changed and requires review",
            )
            .values(&before.path, &after.path),
        );
    }
    compare_named_set(
        "qualification.gate-requirement",
        &before.name,
        &before.requires,
        &after.requires,
        ChangeKind::Revoke,
        ChangeKind::Grant,
        "requires this lower qualification gate",
        changes,
    );
}

fn compare_invariants(
    before: &[InvariantContract],
    after: &[InvariantContract],
    changes: &mut Vec<ArchitectureChange>,
) {
    let old = invariants_by_id(before);
    let new = invariants_by_id(after);
    for id in old
        .keys()
        .chain(new.keys())
        .copied()
        .collect::<BTreeSet<_>>()
    {
        match (old.get(id), new.get(id)) {
            (None, Some(_)) => changes.push(ArchitectureChange::new(
                ChangeKind::Revoke,
                "invariant",
                id,
                "new evidence-backed invariant was declared",
            )),
            (Some(_), None) => changes.push(ArchitectureChange::new(
                ChangeKind::Grant,
                "invariant",
                id,
                "evidence-backed invariant was removed",
            )),
            (Some(left), Some(right)) => compare_invariant(left, right, changes),
            (None, None) => {}
        }
    }
}

fn compare_invariant(
    before: &InvariantContract,
    after: &InvariantContract,
    changes: &mut Vec<ArchitectureChange>,
) {
    if before.title != after.title || before.document != after.document {
        changes.push(ArchitectureChange::new(
            ChangeKind::Unknown,
            "invariant.definition",
            &before.id,
            "invariant meaning or documentation changed and requires review",
        ));
    }
    compare_named_set(
        "invariant.evidence",
        &before.id,
        &before.evidence,
        &after.evidence,
        ChangeKind::Revoke,
        ChangeKind::Grant,
        "requires this exact evidence",
        changes,
    );
}

fn gates_by_name(gates: &[GateContract]) -> BTreeMap<&str, &GateContract> {
    gates
        .iter()
        .map(|gate| (gate.name.as_str(), gate))
        .collect()
}

fn invariants_by_id(invariants: &[InvariantContract]) -> BTreeMap<&str, &InvariantContract> {
    invariants
        .iter()
        .map(|invariant| (invariant.id.as_str(), invariant))
        .collect()
}

#[cfg(test)]
#[path = "evidence_test.rs"]
mod evidence_test;
