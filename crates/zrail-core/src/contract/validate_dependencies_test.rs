//! Dependency identity attestations remain narrow and reviewable.

use super::super::validate_fixture_test::minimal_contract;
use crate::CrateRootSource;

#[test]
fn crate_root_attestations_require_unique_packages_valid_roots_and_reasons() {
    let mut contract = minimal_contract();
    contract.dependencies.crate_roots = vec![
        crate::CrateRootContract {
            package: "runtime".into(),
            root: "runtime_core".into(),
            reason: "The registry package exposes this library target.".into(),
            source: registry("1"),
        },
        crate::CrateRootContract {
            package: "runtime".into(),
            root: "not::a::root".into(),
            reason: String::new(),
            source: registry("1"),
        },
        crate::CrateRootContract {
            package: "keyword-root".into(),
            root: "r#self".into(),
            reason: "Raw path keywords cannot identify dependency crates.".into(),
            source: registry("1"),
        },
    ];

    let error = super::super::validate::validate_contract(&contract)
        .expect_err("reject ambiguous attestations");
    let message = error.to_string();

    assert!(message.contains("duplicate dependency crate-root attestation"));
    assert!(message.contains("must be one normalized Rust crate identifier"));
    assert!(message.contains("requires a reason"));
}

#[test]
fn one_package_can_have_distinct_exact_source_attestations() {
    let mut contract = minimal_contract();
    contract.dependencies.crate_roots = vec![
        crate::CrateRootContract {
            package: "runtime".into(),
            root: "registry_runtime".into(),
            reason: "Reviewed registry metadata.".into(),
            source: registry("1"),
        },
        crate::CrateRootContract {
            package: "runtime".into(),
            root: "git_runtime".into(),
            reason: "Reviewed Git metadata.".into(),
            source: CrateRootSource::Git {
                repository: "https://example.invalid/runtime".into(),
                branch: None,
                tag: None,
                rev: Some("abc123".into()),
                requirement: None,
            },
        },
    ];

    super::super::validate::validate_contract(&contract)
        .expect("exact dependency sources are independent authorities");
}

#[test]
fn repository_macro_provenance_is_not_dependency_crate_root_authority() {
    let mut contract = minimal_contract();
    contract.dependencies.crate_roots = vec![crate::CrateRootContract {
        package: "runtime".into(),
        root: "runtime".into(),
        reason: "Repository macro provenance is a different authority kind.".into(),
        source: CrateRootSource::Repository {
            package: "runtime".into(),
            directory: "crates/runtime".into(),
        },
    }];

    let error = super::super::validate::validate_contract(&contract)
        .expect_err("reject repository provenance for dependency crate roots");
    assert!(
        error
            .to_string()
            .contains("may not select repository macro provenance")
    );
}

fn registry(requirement: &str) -> CrateRootSource {
    CrateRootSource::Registry {
        registry: None,
        index: None,
        requirement: requirement.into(),
    }
}
