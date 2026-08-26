//! Exact type policy permission changes remain review-visible.

use crate::{
    ChangeKind, CloneCopyPolicy, DuplicationTrait, PolicyReachability, RustFieldContract,
    RustTypeContract, RustTypeKind, TypeProhibition, compare_architecture,
};

use super::compare_fixture_test::contract_with_hard_limit;

#[test]
fn adding_and_removing_exact_type_policy_are_directed() {
    let open = contract_with_hard_limit(300);
    let mut governed = open.clone();
    governed.source.rust.types.push(authority());

    let added = compare_architecture(&open, None, &governed, None);
    let removed = compare_architecture(&governed, None, &open, None);

    assert_change(&added, ChangeKind::Revoke, "rust.type-policy");
    assert_change(&removed, ChangeKind::Grant, "rust.type-policy");
}

#[test]
fn clone_copy_closure_and_written_syntax_changes_are_directed() {
    let mut open = contract_with_hard_limit(300);
    open.source.rust.types.push(type_policy());
    let mut strict = open.clone();
    strict.source.rust.types[0].clone_copy = CloneCopyPolicy::Forbidden;
    strict.source.rust.duplication.deny_imports = vec![DuplicationTrait::Clone];

    let tightened = compare_architecture(&open, None, &strict, None);
    let relaxed = compare_architecture(&strict, None, &open, None);

    assert_change(
        &tightened,
        ChangeKind::Revoke,
        "rust.type-policy.clone-copy",
    );
    assert_change(&tightened, ChangeKind::Revoke, "rust.duplication.import");
    assert_change(&relaxed, ChangeKind::Grant, "rust.type-policy.clone-copy");
    assert_change(&relaxed, ChangeKind::Grant, "rust.duplication.import");
}

#[test]
fn exact_representation_changes_are_unknown() {
    let mut before = contract_with_hard_limit(300);
    before.source.rust.types.push(authority());
    let mut after = before.clone();
    after.source.rust.types[0].fields.as_mut().expect("fields")[0].type_identity = "u128".into();

    let report = compare_architecture(&before, None, &after, None);

    assert_change(&report, ChangeKind::Unknown, "rust.type-policy.shape");
}

#[test]
fn removing_redundant_deny_from_clone_copy_closure_is_not_a_grant() {
    let mut before = contract_with_hard_limit(300);
    let mut policy = type_policy();
    policy.clone_copy = CloneCopyPolicy::Forbidden;
    policy.deny = vec![TypeProhibition::ImplClone];
    before.source.rust.types.push(policy);
    let mut after = before.clone();
    after.source.rust.types[0].deny.clear();

    let report = compare_architecture(&before, None, &after, None);

    assert!(
        report
            .changes
            .iter()
            .all(|change| change.rail != "rust.type-policy.prohibition"),
        "redundant authored deny changed effective policy: {:?}",
        report.changes
    );
}

fn type_policy() -> RustTypeContract {
    RustTypeContract {
        name: "permit".into(),
        identity: "crate::authority::Permit".into(),
        path: "src/authority.rs".into(),
        kind: RustTypeKind::Type,
        reachability: PolicyReachability::Production,
        deny: vec![TypeProhibition::ImplClone],
        clone_copy: CloneCopyPolicy::Allow,
        visibility: None,
        leaf_module: None,
        fields: None,
        reason: "Carries non-duplicable authority.".into(),
    }
}

fn authority() -> RustTypeContract {
    let mut policy = type_policy();
    policy.kind = RustTypeKind::AuthorityToken;
    policy.clone_copy = CloneCopyPolicy::Forbidden;
    policy.visibility = Some("private".into());
    policy.leaf_module = Some(true);
    policy.fields = Some(vec![RustFieldContract {
        name: "epoch".into(),
        type_identity: "u64".into(),
        visibility: "private".into(),
    }]);
    policy
}

fn assert_change(report: &crate::DiffReport, kind: ChangeKind, rail: &str) {
    assert!(
        report
            .changes
            .iter()
            .any(|change| change.kind == kind && change.rail == rail),
        "missing {kind:?} {rail:?}: {:?}",
        report.changes
    );
}
