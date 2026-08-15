//! Path explanations expose their invariant relationships.

use zrail_core::{GateContract, GateKind, InvariantContract, InvariantStatus};

use super::mentions_path;

#[test]
fn connects_test_document_and_gate_paths_to_invariants() {
    let gates = [GateContract {
        name: "check".into(),
        kind: GateKind::Local,
        path: "scripts/check".into(),
        requires: Vec::new(),
        reason: "test".into(),
    }];
    let invariant = InvariantContract {
        id: "ARCH-01".into(),
        title: "Architecture".into(),
        status: InvariantStatus::Enforced,
        document: "docs/architecture.md#arch-01".into(),
        evidence: vec![
            "rust-test:src/architecture_test.rs::works".into(),
            "gate:check".into(),
        ],
    };

    for path in [
        "docs/architecture.md",
        "src/architecture_test.rs",
        "scripts/check",
    ] {
        assert!(mentions_path(&gates, &invariant, path));
    }
    assert!(!mentions_path(&gates, &invariant, "src/lib.rs"));
}
