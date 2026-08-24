//! Contract limits account for every evidence-graph node and edge.

use crate::{
    GateContract, GateKind, InvariantContract, InvariantStatus, TestExecutionIdentity,
    TestMirrorContract,
};

use super::{evidence_items, test_mirror_items};

#[test]
fn evidence_nodes_and_edges_consume_the_contract_limit() {
    let gates = [GateContract {
        name: "ci".into(),
        kind: GateKind::Ci,
        path: "ci/check".into(),
        inputs: vec!["ci/action.yml".into()],
        requires: vec!["local".into(), "archive".into()],
        reason: "canonical CI gate".into(),
    }];
    let invariants = [InvariantContract {
        id: "QUAL-01".into(),
        title: "Qualification remains connected".into(),
        status: InvariantStatus::Enforced,
        document: "docs/design.md#qualification".into(),
        evidence: vec!["test:qualification".into(), "gate:ci".into()],
    }];

    assert_eq!(evidence_items(&gates, &invariants), 7);
}

#[test]
fn test_mirror_inputs_and_features_consume_the_contract_limit() {
    let mirrors = [TestMirrorContract {
        production: "src/state.rs".into(),
        test: "tests/state.rs".into(),
        name: "works".into(),
        receipt: "evidence/state.json".into(),
        inputs: vec![
            "Cargo.lock".into(),
            "Cargo.toml".into(),
            "src/shared.rs".into(),
        ],
        execution: TestExecutionIdentity {
            command: "cargo test works".into(),
            package: "state".into(),
            default_features: false,
            features: vec!["a".into(), "b".into()],
            target: "x86_64-unknown-linux-gnu".into(),
            toolchain: "rustc 1.90.0".into(),
        },
        reason: "Exact state behavior".into(),
    }];

    assert_eq!(test_mirror_items(&mirrors), 5);
}
