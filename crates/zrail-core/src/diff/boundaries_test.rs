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
