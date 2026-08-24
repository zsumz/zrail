//! Evidence and gate changes are classified by permission direction.

use crate::{
    ChangeKind, GateContract, GateKind, InvariantContract, InvariantStatus, TestMirrorContract,
};

use super::{compare_gates, compare_invariants, compare_test_mirrors};

#[test]
fn exact_test_mirror_changes_preserve_permission_direction() {
    let mirror = test_mirror("tests/state_test.rs", "state_transitions");
    let mut added = Vec::new();
    compare_test_mirrors(&[], std::slice::from_ref(&mirror), &mut added);
    assert_eq!(added[0].kind, ChangeKind::Revoke);

    let mut removed = Vec::new();
    compare_test_mirrors(std::slice::from_ref(&mirror), &[], &mut removed);
    assert_eq!(removed[0].kind, ChangeKind::Grant);

    let changed = test_mirror("tests/new_state_test.rs", "state_transitions");
    let mut replaced = Vec::new();
    compare_test_mirrors(&[mirror], &[changed], &mut replaced);
    assert_eq!(replaced[0].kind, ChangeKind::Unknown);
}

#[test]
fn adding_graph_nodes_revokes_permission_and_removing_them_grants_it() {
    let gates = vec![gate(Vec::new())];
    let invariants = vec![invariant(vec![
        "rust-test:src/check_test.rs::works".into(),
        "gate:check".into(),
    ])];
    let mut added = Vec::new();
    compare_gates(&[], &gates, &mut added);
    compare_invariants(&[], &invariants, &mut added);
    assert!(added.iter().all(|change| change.kind == ChangeKind::Revoke));

    let mut removed = Vec::new();
    compare_gates(&gates, &[], &mut removed);
    compare_invariants(&invariants, &[], &mut removed);
    assert!(
        removed
            .iter()
            .all(|change| change.kind == ChangeKind::Grant)
    );
}

#[test]
fn disconnecting_evidence_or_a_required_gate_is_a_grant() {
    let mut changes = Vec::new();
    compare_gates(
        &[gate(vec!["unit".into()])],
        &[gate(Vec::new())],
        &mut changes,
    );
    compare_invariants(
        &[invariant(vec![
            "rust-test:src/check_test.rs::works".into(),
            "gate:check".into(),
        ])],
        &[invariant(vec!["gate:check".into()])],
        &mut changes,
    );
    assert_eq!(
        changes
            .iter()
            .filter(|change| change.kind == ChangeKind::Grant)
            .count(),
        2
    );
}

#[test]
fn adding_gate_inputs_revokes_permission_and_removing_them_grants_it() {
    let before = gate(Vec::new());
    let mut after = before.clone();
    after.inputs.push("scripts/structure-check".into());

    let mut added = Vec::new();
    compare_gates(
        std::slice::from_ref(&before),
        std::slice::from_ref(&after),
        &mut added,
    );
    assert_eq!(added.len(), 1);
    assert_eq!(added[0].kind, ChangeKind::Revoke);
    assert_eq!(added[0].rail, "qualification.gate-input");

    let mut removed = Vec::new();
    compare_gates(
        std::slice::from_ref(&after),
        std::slice::from_ref(&before),
        &mut removed,
    );
    assert_eq!(removed.len(), 1);
    assert_eq!(removed[0].kind, ChangeKind::Grant);
}

fn gate(requires: Vec<String>) -> GateContract {
    GateContract {
        name: "check".into(),
        kind: GateKind::Local,
        path: "scripts/check".into(),
        inputs: Vec::new(),
        requires,
        reason: "Canonical qualification".into(),
    }
}

fn invariant(evidence: Vec<String>) -> InvariantContract {
    InvariantContract {
        id: "ARCH-01".into(),
        title: "Architecture qualifies".into(),
        status: InvariantStatus::Enforced,
        document: "docs/architecture.md#arch-01".into(),
        evidence,
    }
}

fn test_mirror(test: &str, name: &str) -> TestMirrorContract {
    TestMirrorContract {
        production: "src/state.rs".into(),
        test: test.into(),
        name: name.into(),
        receipt: "evidence/state.json".into(),
        reason: "Exact state transition coverage".into(),
    }
}
