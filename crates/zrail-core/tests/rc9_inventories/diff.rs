//! Protected review follows accepted states for per-trait type selection and exact pairs.

use crate::{ChangeKind, Contract, RustInventorySubject};

use super::{contract, kinds};

fn selected(assertion: &str, filters: &str) -> Contract {
    let mut contract = contract(assertion);
    contract.source.rust.inventories[0].subject = toml::from_str(&format!(
        "kind='written-trait-impls'\nnames=['Source','SlotTransport']\n{filters}"
    ))
    .expect("subject");
    contract
}

#[test]
fn type_filter_expansion_obeys_positive_and_forbidden_quantifiers() {
    for (assertion, expansion) in [
        ("kind='count',minimum=1", ChangeKind::Grant),
        ("kind='count',maximum=0", ChangeKind::Revoke),
        ("kind='exact-counts',counts=[]", ChangeKind::Revoke),
        ("kind='exact-owners',owners=[]", ChangeKind::Revoke),
        ("kind='count',minimum=1,maximum=1", ChangeKind::Unknown),
    ] {
        let before = selected(assertion, "implementing_types={Source=['State']}");
        for filters in ["", "implementing_types={Source=['State','Other']}"] {
            let after = selected(assertion, filters);
            assert_eq!(kinds(&before, &after), [expansion]);
            let reverse = match expansion {
                ChangeKind::Grant => ChangeKind::Revoke,
                ChangeKind::Revoke => ChangeKind::Grant,
                _ => ChangeKind::Unknown,
            };
            assert_eq!(kinds(&after, &before), [reverse]);
        }
    }
}

#[test]
fn same_count_trait_or_type_substitutions_are_grants_and_revocations() {
    let assertion = "kind='exact-owners',owners=[{path='src/lib.rs',name='Source for State'}]";
    let before = selected(assertion, "");
    for changed in [
        assertion.replace("Source for State", "Source for Other"),
        assertion.replace("Source for State", "SlotTransport for State"),
    ] {
        assert_eq!(
            kinds(&before, &selected(&changed, "")),
            [ChangeKind::Grant, ChangeKind::Revoke]
        );
    }
    let before = selected(
        "kind='count',maximum=0",
        "implementing_types={Source=['State']}",
    );
    let after = selected(
        "kind='count',maximum=0",
        "implementing_types={Source=['Other']}",
    );
    assert_eq!(kinds(&before, &after), [ChangeKind::Unknown]);
}

#[test]
fn type_filter_order_is_neutral_and_kind_changes_remain_unknown() {
    let before = selected(
        "kind='exact-owners',owners=[]",
        "implementing_types={Source=['State','Other'],SlotTransport=['Z','A']}",
    );
    let mut after = before.clone();
    after.source.rust.inventories[0].subject.canonicalize();
    assert!(kinds(&before, &after).is_empty());
    let encoded = toml::to_string(&after.source.rust.inventories[0].subject).expect("canonical");
    assert!(encoded.contains("Source = [\"Other\", \"State\"]"));
    after.source.rust.inventories[0].subject = RustInventorySubject::WrittenMethods {
        names: vec!["Source".into(), "SlotTransport".into()],
    };
    assert_eq!(kinds(&before, &after), [ChangeKind::Unknown]);
}
