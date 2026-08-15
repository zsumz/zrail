//! Contract limits account for every evidence-graph node and edge.

use crate::{GateContract, GateKind, InvariantContract, InvariantStatus};

use super::evidence_items;

#[test]
fn evidence_nodes_and_edges_consume_the_contract_limit() {
    let gates = [GateContract {
        name: "ci".into(),
        kind: GateKind::Ci,
        path: "ci/check".into(),
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

    assert_eq!(evidence_items(&gates, &invariants), 6);
}
