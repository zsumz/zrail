//! Protected review retains exact quantities and accounts for each quantifier's scope semantics.

use crate::{ChangeKind, Contract, RustInventoryRule, compare_architecture};

fn contract(assertion: &str) -> Contract {
    let mut contract = super::super::compare_fixture_test::contract_with_hard_limit(300);
    contract.source.rust.inventories.push(toml::from_str::<RustInventoryRule>(&format!(
        "name='methods'\nreason='Reviewed.'\ninclude=['src/**/*.rs']\nworld='authored'\nsubject={{kind='written-methods',names=['poll','wake']}}\nassertion={{{assertion}}}"
    )).expect("contract"));
    contract
}

fn exact(count: usize) -> String {
    format!("kind='exact-counts',counts=[{{path='src/lib.rs',name='poll',count={count}}}]")
}

fn kinds(before: &Contract, after: &Contract) -> Vec<ChangeKind> {
    let report = compare_architecture(before, None, after, None);
    let kinds = report
        .changes
        .iter()
        .map(|change| {
            assert_eq!(change.rail, "source.rust.inventories");
            assert_eq!(change.subject, "rust:inventory:methods");
            change.kind
        })
        .collect::<Vec<_>>();
    if kinds.contains(&ChangeKind::Grant) || kinds.contains(&ChangeKind::Unknown) {
        assert!(report.denies_grants());
    }
    kinds
}

#[test]
fn exact_changes_in_both_numeric_directions_and_same_quantity_substitutions_are_protected() {
    for (before, after) in [
        (exact(1), exact(2)),
        (exact(2), exact(1)),
        (exact(1), exact(1).replace("poll", "wake")),
        (exact(1), exact(1).replace("lib.rs", "child.rs")),
        (
            "kind='count',minimum=1,maximum=1".into(),
            "kind='count',minimum=0,maximum=0".into(),
        ),
    ] {
        assert_eq!(
            kinds(&contract(&before), &contract(&after)),
            [ChangeKind::Grant, ChangeKind::Revoke]
        );
    }
    let before = contract(&exact(1));
    let after = contract("kind='count',minimum=1,maximum=1");
    assert_eq!(kinds(&before, &after), [ChangeKind::Grant]);
    assert_eq!(kinds(&after, &before), [ChangeKind::Revoke]);
}

#[test]
fn bounds_removal_and_scope_expansion_follow_the_required_quantifier() {
    let before = contract("kind='count',minimum=1,maximum=2");
    for assertion in [
        "kind='count',minimum=1",
        "kind='count',maximum=2",
        "kind='count',minimum=1,maximum=3",
    ] {
        let after = contract(assertion);
        assert_eq!(kinds(&before, &after), [ChangeKind::Grant]);
        assert_eq!(kinds(&after, &before), [ChangeKind::Revoke]);
    }
    for (assertion, expanded) in [
        ("kind='count',minimum=1", ChangeKind::Grant),
        ("kind='count',maximum=0", ChangeKind::Revoke),
        ("kind='exact-counts',counts=[]", ChangeKind::Revoke),
        ("kind='count',minimum=1,maximum=1", ChangeKind::Unknown),
    ] {
        let before = contract(assertion);
        let mut after = before.clone();
        after.source.rust.inventories[0]
            .include
            .push("tests/**/*.rs".into());
        assert_eq!(kinds(&before, &after), [expanded]);
    }
    let mut removed = before.clone();
    removed.source.rust.inventories.clear();
    assert_eq!(kinds(&before, &removed), [ChangeKind::Grant]);
    assert_eq!(kinds(&removed, &before), [ChangeKind::Revoke]);
}

#[test]
fn sets_ignore_order_and_unproved_selection_changes_remain_protected() {
    let before = contract("kind='exact-counts',counts=[]");
    let after = contract("kind='count',maximum=0");
    assert!(kinds(&before, &after).is_empty());
    let mut reordered = before.clone();
    let crate::RustInventorySubject::WrittenMethods { names } =
        &mut reordered.source.rust.inventories[0].subject;
    names.reverse();
    assert!(kinds(&before, &reordered).is_empty());
    reordered.source.rust.inventories[0].include = vec!["other/**/*.rs".into()];
    assert_eq!(kinds(&before, &reordered), [ChangeKind::Unknown]);
}
