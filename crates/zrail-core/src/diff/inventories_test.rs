//! Protected review retains exact quantities and accounts for each quantifier's scope semantics.

use crate::{ChangeKind, Contract, RustInventoryRule, compare_architecture};

#[path = "inventory_impls_test.rs"]
mod impls_test;

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
        ("kind='exact-owners',owners=[]", ChangeKind::Revoke),
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
fn ownership_and_quantity_review_matches_independent_accepted_states() {
    type Acceptance = fn(usize, usize) -> bool;
    let cases: &[(&str, Acceptance)] = &[
        ("kind='count',maximum=0", |a, b| a + b == 0),
        ("kind='count',minimum=1", |a, b| a + b >= 1),
        ("kind='count',minimum=1,maximum=2", |a, b| {
            (1..=2).contains(&(a + b))
        }),
        ("kind='exact-counts',counts=[]", |a, b| a == 0 && b == 0),
        (
            "kind='exact-counts',counts=[{path='src/lib.rs',name='poll',count=1}]",
            |a, b| a == 1 && b == 0,
        ),
        (
            "kind='exact-counts',counts=[{path='src/lib.rs',name='poll',count=2}]",
            |a, b| a == 2 && b == 0,
        ),
        ("kind='exact-owners',owners=[]", |a, b| a == 0 && b == 0),
        (
            "kind='exact-owners',owners=[{path='src/lib.rs',name='poll'}]",
            |a, b| a > 0 && b == 0,
        ),
        (
            "kind='exact-owners',owners=[{path='src/child.rs',name='wake'}]",
            |a, b| a == 0 && b > 0,
        ),
        (
            "kind='exact-owners',owners=[{path='src/lib.rs',name='poll'},{path='src/child.rs',name='wake'}]",
            |a, b| a > 0 && b > 0,
        ),
    ];
    for (before, old_accepts) in cases {
        for (after, new_accepts) in cases {
            let changes = kinds(&contract(before), &contract(after));
            let states = || (0..=3).flat_map(|a| (0..=3).map(move |b| (a, b)));
            assert_eq!(
                changes.contains(&ChangeKind::Grant),
                states().any(|(a, b)| new_accepts(a, b) && !old_accepts(a, b)),
                "grant: {before} -> {after}"
            );
            assert_eq!(
                changes.contains(&ChangeKind::Revoke),
                states().any(|(a, b)| old_accepts(a, b) && !new_accepts(a, b)),
                "revoke: {before} -> {after}"
            );
        }
    }
}

#[test]
fn exact_owner_order_is_neutral_but_removal_and_scope_narrowing_are_protected() {
    let before = contract(
        "kind='exact-owners',owners=[{path='src/lib.rs',name='poll'},{path='src/child.rs',name='wake'}]",
    );
    let mut after = before.clone();
    let crate::RustInventoryAssertion::ExactOwners { owners } =
        &mut after.source.rust.inventories[0].assertion
    else {
        panic!("owners")
    };
    owners.reverse();
    assert!(kinds(&before, &after).is_empty());
    after.source.rust.inventories[0]
        .exclude
        .push("src/unused.rs".into());
    assert_eq!(kinds(&before, &after), [ChangeKind::Grant]);
    after.source.rust.inventories.clear();
    assert_eq!(kinds(&before, &after), [ChangeKind::Grant]);
}

#[test]
fn sets_ignore_order_and_unproved_selection_changes_remain_protected() {
    let before = contract("kind='exact-counts',counts=[]");
    let after = contract("kind='count',maximum=0");
    assert!(kinds(&before, &after).is_empty());
    let mut reordered = before.clone();
    let crate::RustInventorySubject::WrittenMethods { names } =
        &mut reordered.source.rust.inventories[0].subject
    else {
        panic!("written methods")
    };
    names.reverse();
    assert!(kinds(&before, &reordered).is_empty());
    reordered.source.rust.inventories[0].include = vec!["other/**/*.rs".into()];
    assert_eq!(kinds(&before, &reordered), [ChangeKind::Unknown]);
}

#[test]
fn expression_path_scope_quantities_and_kind_changes_remain_protected() {
    use crate::RustInventorySubject::WrittenExpressionPaths;
    for (assertion, expanded) in [
        ("kind='count',maximum=0", ChangeKind::Revoke),
        ("kind='count',minimum=1", ChangeKind::Grant),
    ] {
        let methods = contract(assertion);
        let mut before = methods.clone();
        before.source.rust.inventories[0].subject = WrittenExpressionPaths {
            suffixes: vec!["State::poll".into()],
        };
        assert_eq!(kinds(&methods, &before), [ChangeKind::Unknown]);
        assert_eq!(kinds(&before, &methods), [ChangeKind::Unknown]);
        let mut after = before.clone();
        after.source.rust.inventories[0].subject = WrittenExpressionPaths {
            suffixes: vec!["State::wake".into(), "State::poll".into()],
        };
        assert_eq!(kinds(&before, &after), [expanded]);
        let mut reordered = after.clone();
        reordered.source.rust.inventories[0].subject.canonicalize();
        assert!(kinds(&after, &reordered).is_empty());
    }
    let mut before = contract(&exact(1).replace("poll", "State::poll"));
    before.source.rust.inventories[0].subject = WrittenExpressionPaths {
        suffixes: vec!["State::poll".into(), "State::wake".into()],
    };
    let mut after = before.clone();
    let crate::RustInventoryAssertion::ExactCounts { counts } =
        &mut after.source.rust.inventories[0].assertion
    else {
        panic!("exact")
    };
    counts[0].name = "State::wake".into();
    assert_eq!(
        kinds(&before, &after),
        [ChangeKind::Grant, ChangeKind::Revoke]
    );
}

#[test]
fn written_path_membership_is_distinct_authority_with_reviewed_scope_semantics() {
    for (assertion, expanded) in [
        ("kind='count',maximum=0", ChangeKind::Revoke),
        ("kind='count',minimum=1", ChangeKind::Grant),
        ("kind='exact-owners',owners=[]", ChangeKind::Revoke),
    ] {
        let before = contract(assertion);
        let mut after = before.clone();
        after.source.rust.inventories[0].subject =
            crate::RustInventorySubject::WrittenPathsContaining {
                names: vec!["ConnectionSet".into()],
            };
        assert_eq!(kinds(&before, &after), [ChangeKind::Unknown]);
        assert_eq!(kinds(&after, &before), [ChangeKind::Unknown]);
        let before = after.clone();
        after.source.rust.inventories[0].subject =
            crate::RustInventorySubject::WrittenPathsContaining {
                names: vec!["ConnectionSet".into(), "Alias".into()],
            };
        assert_eq!(kinds(&before, &after), [expanded]);
        let mut reordered = after.clone();
        reordered.source.rust.inventories[0].subject.canonicalize();
        assert!(kinds(&after, &reordered).is_empty());
    }
}

#[test]
fn changing_rename_destinations_or_source_selection_requires_protected_review() {
    let mut before =
        contract("kind='exact-owners',owners=[{path='src/lib.rs',name='Source as Alias'}]");
    before.source.rust.inventories[0].subject = crate::RustInventorySubject::WrittenImportRenames {
        names: vec!["Source".into()],
    };
    let mut after = before.clone();
    let crate::RustInventoryAssertion::ExactOwners { owners } =
        &mut after.source.rust.inventories[0].assertion
    else {
        panic!("owners")
    };
    owners[0].name = "Source as Other".into();
    assert_eq!(
        kinds(&before, &after),
        [ChangeKind::Grant, ChangeKind::Revoke]
    );
    before.source.rust.inventories[0].assertion = crate::RustInventoryAssertion::Count {
        minimum: 0,
        maximum: Some(0),
    };
    after = before.clone();
    after.source.rust.inventories[0].subject = crate::RustInventorySubject::WrittenImportRenames {
        names: vec!["Source".into(), "State".into()],
    };
    assert_eq!(kinds(&before, &after), [ChangeKind::Revoke]);
    assert_eq!(kinds(&after, &before), [ChangeKind::Grant]);
    after.source.rust.inventories[0].subject = crate::RustInventorySubject::WrittenMethods {
        names: vec!["Source".into()],
    };
    assert_eq!(kinds(&before, &after), [ChangeKind::Unknown]);
}
