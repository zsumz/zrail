//! External crate-root attestations never override stronger local evidence.

use zrail_core::{CrateRootContract, CrateRootSource};

use super::super::{
    CargoWorkspace, CrateRootAuthority, Dependency, DependencyKind, DependencySource, Package,
};
use super::apply_attestations;

#[test]
fn attestations_resolve_only_uninspected_external_packages() {
    let mut cargo = CargoWorkspace {
        declared_members: Vec::new(),
        observed_members: Vec::new(),
        packages: vec![Package {
            name: "app".into(),
            directory: ".".into(),
            dependencies: vec![dependency(
                "runtime",
                "tokio",
                CrateRootAuthority::Unresolved,
            )],
            targets: Vec::new(),
        }],
        resolution_overrides: Vec::new(),
    };

    apply_attestations(
        &mut cargo,
        &[CrateRootContract {
            package: "tokio".into(),
            root: "runtime_core".into(),
            reason: "reviewed registry metadata".into(),
            source: CrateRootSource::Registry {
                registry: None,
                index: None,
                requirement: "1".into(),
            },
        }],
    );

    let dependency = &cargo.packages[0].dependencies[0];
    assert_eq!(dependency.crate_root, "runtime_core");
    assert_eq!(
        dependency.crate_root_authority,
        CrateRootAuthority::Attested
    );
}

#[test]
fn same_name_attestations_match_only_their_exact_source() {
    let registry = CrateRootContract {
        package: "runtime".into(),
        root: "registry_runtime".into(),
        reason: "reviewed registry metadata".into(),
        source: CrateRootSource::Registry {
            registry: None,
            index: None,
            requirement: "1".into(),
        },
    };
    let git_source = DependencySource::Git {
        repository: "https://example.invalid/runtime".into(),
        branch: None,
        tag: None,
        rev: Some("abc123".into()),
        requirement: None,
    };

    assert!(!super::attestation_matches(
        &registry,
        "runtime",
        &git_source
    ));
    assert!(super::attestation_matches(
        &registry,
        "runtime",
        &DependencySource::Registry {
            registry: None,
            index: None,
            requirement: "1".into(),
        }
    ));
}

fn dependency(alias: &str, name: &str, authority: CrateRootAuthority) -> Dependency {
    Dependency {
        alias: alias.into(),
        name: name.into(),
        explicit_package: false,
        crate_root: alias.into(),
        crate_root_authority: authority,
        kind: DependencyKind::Normal,
        target: None,
        optional: false,
        default_features: true,
        features: Vec::new(),
        source: DependencySource::Registry {
            registry: None,
            index: None,
            requirement: "1".into(),
        },
    }
}
