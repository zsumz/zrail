//! Ownership reachability permission classification.

use crate::{ChangeKind, OwnerContract, OwnerKind, PolicyReachability, compare_architecture};

use crate::diff::compare_fixture_test::contract_with_hard_limit;

#[test]
fn narrowing_owner_evaluation_to_production_is_a_grant() {
    let mut all = contract_with_hard_limit(300);
    all.owners.push(OwnerContract {
        name: "process".into(),
        kind: OwnerKind::Call,
        reachability: PolicyReachability::All,
        within: vec!["src/**".into()],
        selector: "std::process::Command::new".into(),
        mutating_methods: Vec::new(),
        allow: vec!["src/executor.rs".into()],
        reason: "one runtime process owner".into(),
    });
    let mut production = all.clone();
    production.owners[0].reachability = PolicyReachability::Production;

    let narrowed = compare_architecture(&all, None, &production, None);
    let restored = compare_architecture(&production, None, &all, None);

    assert!(
        narrowed.changes.iter().any(|change| {
            change.kind == ChangeKind::Grant && change.rail == "owner.reachability"
        })
    );
    assert!(restored.changes.iter().any(|change| {
        change.kind == ChangeKind::Revoke && change.rail == "owner.reachability"
    }));
}

#[test]
fn adding_and_removing_operation_ownership_has_directional_diff() {
    let before = contract_with_hard_limit(300);
    let mut after = before.clone();
    after.owners.push(OwnerContract {
        name: "transition".into(),
        kind: OwnerKind::MethodName,
        reachability: PolicyReachability::All,
        within: vec!["src/**".into()],
        selector: "transition".into(),
        mutating_methods: Vec::new(),
        allow: vec!["src/state.rs".into()],
        reason: "one written method owner".into(),
    });

    let added = compare_architecture(&before, None, &after, None);
    let removed = compare_architecture(&after, None, &before, None);

    assert!(added.changes.iter().any(|change| {
        change.kind == ChangeKind::Revoke
            && change.rail == "owner"
            && change.subject == "transition"
    }));
    assert!(removed.changes.iter().any(|change| {
        change.kind == ChangeKind::Grant && change.rail == "owner" && change.subject == "transition"
    }));
}

#[test]
fn field_mutation_method_changes_have_directional_diff() {
    let mut before = contract_with_hard_limit(300);
    before.owners.push(OwnerContract {
        name: "values-mutation".into(),
        kind: OwnerKind::FieldMutation,
        reachability: PolicyReachability::All,
        within: vec!["src/**".into()],
        selector: "crate::State::values".into(),
        mutating_methods: vec!["clear".into()],
        allow: vec!["src/state.rs".into()],
        reason: "one values mutation owner".into(),
    });
    let mut after = before.clone();
    after.owners[0].mutating_methods.push("push".into());

    let tightened = compare_architecture(&before, None, &after, None);
    let relaxed = compare_architecture(&after, None, &before, None);

    assert!(tightened.changes.iter().any(|change| {
        change.kind == ChangeKind::Revoke && change.rail == "owner.mutating-method"
    }));
    assert!(relaxed.changes.iter().any(|change| {
        change.kind == ChangeKind::Grant && change.rail == "owner.mutating-method"
    }));
}
