//! Field mutation owners combine structural mutation with declared receiver methods.

use zrail_core::{AnalysisQuality, OwnerContract, OwnerKind, PolicyReachability};

use crate::source::{
    FactNamespace, ObservedFact, SourceOperationFact, SourceOperationKind, SyntaxGuard,
};

use super::operation_matches;

#[test]
fn field_mutation_matches_only_declared_receiver_methods() {
    let owner = owner();

    assert!(operation_matches(
        &owner,
        &operation(SourceOperationKind::FieldReceiverCall, Some("push"))
    ));
    assert!(!operation_matches(
        &owner,
        &operation(SourceOperationKind::FieldReceiverCall, Some("retain"))
    ));
    assert!(operation_matches(
        &owner,
        &operation(SourceOperationKind::FieldWrite, None)
    ));
    assert!(operation_matches(
        &owner,
        &operation(SourceOperationKind::FieldMutableBorrow, None)
    ));
    assert!(!operation_matches(
        &owner,
        &operation(SourceOperationKind::FieldRead, None)
    ));
}

fn owner() -> OwnerContract {
    OwnerContract {
        name: "values-mutation".into(),
        kind: OwnerKind::FieldMutation,
        reachability: PolicyReachability::All,
        within: vec!["src/**".into()],
        selector: "crate::State::values".into(),
        mutating_methods: vec!["clear".into(), "push".into()],
        allow: vec!["src/state.rs".into()],
        reason: "values mutate behind one owner".into(),
    }
}

fn operation(kind: SourceOperationKind, method: Option<&str>) -> SourceOperationFact {
    SourceOperationFact {
        kind,
        identity: ObservedFact {
            name: "crate::State::values".into(),
            written: Some("values".into()),
            canonical: Vec::new(),
            span: None,
            quality: AnalysisQuality::Exact,
            guard: SyntaxGuard::Ordinary,
            lexical_scope: Vec::new(),
            namespace: FactNamespace::Type,
        },
        file_local: false,
        method: method.map(str::to_owned),
        place: None,
    }
}
