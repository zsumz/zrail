//! Field mutation owners combine structural mutation with declared receiver methods.

use zrail_core::{AnalysisQuality, OwnerContract, OwnerKind, PolicyReachability};

use crate::source::{
    FactNamespace, ObservedFact, SourceOperationFact, SourceOperationKind, SyntaxGuard,
};

use super::{operation_matches, selector_matches};

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
    assert!(operation_matches(
        &owner,
        &operation(SourceOperationKind::FieldProjectionWrite, None)
    ));
    assert!(operation_matches(
        &owner,
        &operation(SourceOperationKind::FieldProjectionMutableBorrow, None)
    ));
    assert!(!operation_matches(
        &owner,
        &operation(SourceOperationKind::FieldRead, None)
    ));
}

#[test]
fn opaque_update_fields_match_each_owner_on_the_same_source_type() {
    let mut owner = owner();
    owner.kind = OwnerKind::FieldRead;
    let mut operation = operation(SourceOperationKind::FieldRead, None);
    operation.identity.name = "crate::State::*".into();
    operation.identity.quality = AnalysisQuality::Unresolved;

    assert!(selector_matches(&owner, &operation));
    let mut other = owner;
    other.selector = "crate::Other::values".into();
    assert!(!selector_matches(&other, &operation));
}

#[test]
fn opaque_field_wildcard_does_not_match_non_field_owner() {
    let mut owner = owner();
    owner.kind = OwnerKind::TypeConstruction;
    owner.selector = "crate::State".into();
    let mut operation = operation(SourceOperationKind::TypeConstruction, None);
    operation.identity.name = "crate::*".into();
    operation.identity.quality = AnalysisQuality::Unresolved;

    assert!(!selector_matches(&owner, &operation));
}

#[test]
fn canonical_opaque_field_does_not_fall_back_to_written_root() {
    let mut owner = owner();
    owner.kind = OwnerKind::FieldRead;
    owner.selector = "crate::wire::Ticket::local_secret".into();
    let mut operation = operation(SourceOperationKind::FieldRead, None);
    operation.identity.name = "wire::Ticket::*".into();
    operation.identity.canonical = vec!["wire_model::Ticket::*".into()];
    operation.identity.quality = AnalysisQuality::Unresolved;

    assert!(!selector_matches(&owner, &operation));
    owner.selector = "wire_model::Ticket::external_secret".into();
    assert!(selector_matches(&owner, &operation));
}

#[test]
fn unresolved_field_on_a_known_unrelated_type_does_not_match() {
    let owner = owner();
    let mut operation = operation(SourceOperationKind::FieldReceiverCall, Some("clear"));
    operation.identity.name = "crate::Input::DangerousRawConfigurationProposal::values".into();
    operation.identity.quality = AnalysisQuality::Unresolved;

    assert!(!selector_matches(&owner, &operation));
    operation.identity.name = "<unresolved>::values".into();
    assert!(selector_matches(&owner, &operation));
}

#[test]
fn anchored_relative_field_on_an_unrelated_type_does_not_match() {
    let owner = owner();
    for base in ["super::Input::Variant", "self::Input", "::external::Input"] {
        let mut operation = operation(SourceOperationKind::FieldMutableBorrow, None);
        operation.identity.name = format!("{base}::values");
        operation.identity.quality = AnalysisQuality::Unresolved;
        assert!(!selector_matches(&owner, &operation), "{base}");
    }
}

#[test]
fn canonical_unrelated_base_with_the_same_type_leaf_does_not_match() {
    let owner = owner();
    let mut operation = operation(SourceOperationKind::FieldWrite, None);
    operation.identity.name = "crate::other::State::values".into();
    operation.identity.quality = AnalysisQuality::Unresolved;

    assert!(!selector_matches(&owner, &operation));
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
            implicit_prelude: crate::source::ImplicitPreludeEligibility::Disabled,
            canonical: Vec::new(),
            span: None,
            quality: AnalysisQuality::Exact,
            guard: SyntaxGuard::Ordinary,
            lexical_scope: Vec::new(),
            namespace: FactNamespace::Type,
            generic_shadow: None,
            associated_candidates: Vec::new(),
            inherits_parent_context: true,
        },
        root_lookup: None,
        generic_shadow: None,
        file_local: false,
        subject_origin: crate::source::OperationSubjectOrigin::WrittenPath,
        construction: None,
        construction_proven: false,
        method: method.map(str::to_owned),
        place: None,
        struct_update: None,
        qualified_subject: None,
    }
}
