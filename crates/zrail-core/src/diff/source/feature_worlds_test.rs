//! Exact-world additions tighten while removed feature coverage grants authority.

use crate::{
    CargoFeaturePackageContract, CargoFeatureWorldContract, ChangeKind,
    diff::compare_fixture_test::contract_with_hard_limit,
};

#[test]
fn switching_between_legacy_and_exact_worlds_is_unknown() {
    let before = contract_with_hard_limit(300);
    let mut after = before.clone();
    after.source.rust.feature_worlds = vec![world()];

    let added = crate::compare_architecture(&before, None, &after, None);
    let removed = crate::compare_architecture(&after, None, &before, None);

    assert!(added.changes.iter().any(|change| {
        change.rail == "rust.feature-world.mode" && change.kind == ChangeKind::Unknown
    }));
    assert!(removed.changes.iter().any(|change| {
        change.rail == "rust.feature-world.mode" && change.kind == ChangeKind::Unknown
    }));
}

#[test]
fn changing_selected_features_is_non_monotonic() {
    let mut before = contract_with_hard_limit(300);
    before.source.rust.feature_worlds = vec![world()];
    let mut after = before.clone();
    after.source.rust.feature_worlds[0].packages[0]
        .features
        .push("trace".into());

    let report = crate::compare_architecture(&before, None, &after, None);

    assert!(report.changes.iter().any(|change| {
        change.rail == "rust.feature-world.feature"
            && change.subject == "shipping:core:trace"
            && change.kind == ChangeKind::Unknown
    }));
}

fn world() -> CargoFeatureWorldContract {
    CargoFeatureWorldContract {
        name: "shipping".into(),
        packages: vec![CargoFeaturePackageContract {
            package: "core".into(),
            default_features: false,
            features: vec!["strict".into()],
        }],
        reason: "models the shipping build".into(),
    }
}
