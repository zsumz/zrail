//! Dependency identity attestations remain narrow and reviewable.

use super::super::validate_fixture_test::minimal_contract;

#[test]
fn crate_root_attestations_require_unique_packages_valid_roots_and_reasons() {
    let mut contract = minimal_contract();
    contract.dependencies.crate_roots = vec![
        crate::CrateRootContract {
            package: "runtime".into(),
            root: "runtime_core".into(),
            reason: "The registry package exposes this library target.".into(),
        },
        crate::CrateRootContract {
            package: "runtime".into(),
            root: "not::a::root".into(),
            reason: String::new(),
        },
        crate::CrateRootContract {
            package: "keyword-root".into(),
            root: "r#self".into(),
            reason: "Raw path keywords cannot identify dependency crates.".into(),
        },
    ];

    let error = super::super::validate::validate_contract(&contract)
        .expect_err("reject ambiguous attestations");
    let message = error.to_string();

    assert!(message.contains("duplicate dependency crate-root attestation"));
    assert!(message.contains("must be one normalized Rust crate identifier"));
    assert!(message.contains("requires a reason"));
}
