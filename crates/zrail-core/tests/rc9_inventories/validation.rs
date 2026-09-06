//! Trait inventories require complete written pairs and bounded, explicit type filters.

use crate::{RustInventoryRule, RustInventorySubject};

use super::{rule, validate};

fn selected(identity: &str, filters: &str) -> RustInventoryRule {
    let mut selected = rule(&format!(
        "kind='exact-owners',owners=[{{path='src/lib.rs',name='{identity}'}}]"
    ));
    selected.subject = toml::from_str::<RustInventorySubject>(&format!(
        "kind='written-trait-impls'\nnames=['Source','SlotTransport']\n{filters}"
    ))
    .expect("trait subject");
    selected
}

#[test]
fn exact_impl_identities_keep_both_trait_and_path_type_suffix() {
    for identity in [
        "Source for State",
        "Source for r#State",
        "SlotTransport for _State2",
    ] {
        validate(selected(identity, "")).expect("exact written trait/type pair");
    }
    for identity in [
        "Source",
        "Other for State",
        "Source for *",
        "Source for crate::State",
        "Source for State<T>",
        "Source for &State",
        "Source for _",
        "Source for ",
        "Source for State for Other",
        "Source  for State",
        "r#Source for State",
    ] {
        assert!(validate(selected(identity, "")).is_err(), "{identity}");
    }
    let mut counted = selected("Source for State", "");
    counted.assertion = crate::RustInventoryAssertion::ExactCounts {
        counts: vec![crate::RustInventoryCount {
            path: "src/lib.rs".into(),
            name: "Source for State".into(),
            count: 2,
        }],
    };
    validate(counted).expect("quantity retains both identities");
}

#[test]
fn implementing_type_filters_are_nonempty_exact_sets_of_selected_traits() {
    validate(selected(
        "Source for State",
        "implementing_types={Source=['State','Other']}",
    ))
    .expect("explicit type subset");
    validate(selected(
        "SlotTransport for Other",
        "implementing_types={Source=['State']}",
    ))
    .expect("unfiltered trait selects every path type");
    for filters in [
        "implementing_types={Source=[]}",
        "implementing_types={Source=['Other']}",
        "implementing_types={Source=['State','State']}",
        "implementing_types={Other=['State']}",
        "implementing_types={Source=['State','*']}",
        "implementing_types={Source=['State','crate::Other']}",
        "implementing_types={Source=['State','Other<T>']}",
    ] {
        assert!(validate(selected("Source for State", filters)).is_err());
    }
    let mut oversized = selected("Source for State", "");
    let RustInventorySubject::WrittenTraitImpls {
        implementing_types, ..
    } = &mut oversized.subject
    else {
        panic!("impl subject")
    };
    implementing_types.insert("Source".into(), vec!["State".into()]);
    implementing_types.insert(
        "SlotTransport".into(),
        (0..128).map(|index| format!("Type{index}")).collect(),
    );
    assert!(
        validate(oversized).is_err(),
        "aggregate bound across traits"
    );
}

#[test]
fn unsupported_impl_semantics_and_duplicate_filter_keys_fail_parsing() {
    for extra in [
        "polarity='positive'",
        "resolve=true",
        "implementing_types=['State']",
        "implementing_types={Source='State'}",
        "implementing_types={Source=['State'],Source=['Other']}",
    ] {
        assert!(
            toml::from_str::<RustInventorySubject>(&format!(
                "kind='written-trait-impls'\nnames=['Source']\n{extra}"
            ))
            .is_err(),
            "{extra}"
        );
    }
}
