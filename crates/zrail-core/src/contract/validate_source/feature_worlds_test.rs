//! Feature-world validation rejects partial-looking authored identities early.

use crate::contract::{
    CargoFeaturePackageContract, CargoFeatureWorldContract, validate::validate_contract,
    validate_fixture_test::minimal_contract,
};

#[test]
fn accepts_unique_named_world_package_selections() {
    let mut contract = minimal_contract();
    contract.source.rust.feature_worlds = vec![world()];

    validate_contract(&contract).expect("valid feature world");
}

#[test]
fn rejects_duplicate_packages_features_and_unstable_names() {
    let mut contract = minimal_contract();
    let mut world = world();
    world.name = "not a name".into();
    world.packages[0].features = vec!["strict".into(), "strict".into()];
    world.packages.push(world.packages[0].clone());
    contract.source.rust.feature_worlds = vec![world];

    let error = validate_contract(&contract)
        .expect_err("invalid feature world")
        .to_string();

    assert!(error.contains("not a stable identifier"));
    assert!(error.contains("repeats package"));
    assert!(error.contains("repeats feature"));
}

fn world() -> CargoFeatureWorldContract {
    CargoFeatureWorldContract {
        name: "shipping".into(),
        packages: vec![CargoFeaturePackageContract {
            package: "core".into(),
            default_features: false,
            features: vec!["strict".into()],
        }],
        reason: "models the shipping workspace build".into(),
    }
}
