//! Workspace feature worlds prove complete maps and exact propagation.

use std::collections::BTreeMap;

use toml::Value;

use super::{FeaturePackageSelection, FeatureWorldSpec, resolve_feature_worlds};
use crate::cargo::{
    CargoWorkspace, CrateRootAuthority, Dependency, DependencyKind, DependencySource, Package,
    PackageFeatureSet,
};

#[test]
fn resolves_optional_workspace_dependency_features_to_a_fixed_point() {
    let cargo = workspace(None);
    let worlds = resolve_feature_worlds(&cargo, &[world()]).expect("resolve exact world");

    assert_eq!(worlds[0].name, "shipping");
    assert_eq!(worlds[0].packages["app"].active, ["runtime"]);
    assert_eq!(worlds[0].packages["core"].active, ["trace"]);
}

#[test]
fn rejects_incomplete_package_maps() {
    let mut spec = world();
    spec.packages.pop();

    let error = resolve_feature_worlds(&workspace(None), &[spec]).expect_err("reject partial map");

    assert!(error.contains("must select every workspace package"));
    assert!(error.contains("core"));
}

#[test]
fn rejects_target_conditional_feature_propagation() {
    let error = resolve_feature_worlds(&workspace(Some("cfg(unix)")), &[world()])
        .expect_err("reject target ambiguity");

    assert!(error.contains("is not exact"));
    assert!(error.contains("target-conditional"));
}

fn world() -> FeatureWorldSpec {
    FeatureWorldSpec {
        name: "shipping".into(),
        packages: vec![
            FeaturePackageSelection {
                package: "app".into(),
                default_features: false,
                features: vec!["runtime".into()],
            },
            FeaturePackageSelection {
                package: "core".into(),
                default_features: false,
                features: Vec::new(),
            },
        ],
    }
}

fn workspace(target: Option<&str>) -> CargoWorkspace {
    let dependency = Dependency {
        alias: "core".into(),
        name: "core".into(),
        explicit_package: false,
        crate_root: "core".into(),
        crate_root_authority: CrateRootAuthority::DeclaredAlias,
        kind: DependencyKind::Normal,
        target: target.map(str::to_owned),
        optional: true,
        default_features: false,
        features: Vec::new(),
        source: DependencySource::WorkspaceMember {
            directory: "core".into(),
            requirement: None,
        },
    };
    let app_features = features(
        "[features]\nruntime = ['dep:core', 'core/trace']\n",
        std::slice::from_ref(&dependency),
    );
    let core_features = features("[features]\ntrace = []\n", &[]);
    CargoWorkspace {
        declared_members: vec!["app".into(), "core".into()],
        observed_members: vec!["app".into(), "core".into()],
        packages: vec![
            package("app", vec![dependency]),
            package("core", Vec::new()),
        ],
        package_features: BTreeMap::from([
            ("app".into(), app_features),
            ("core".into(), core_features),
        ]),
        authority_surfaces: Vec::new(),
        manifest_scopes: BTreeMap::new(),
    }
}

fn package(name: &str, dependencies: Vec<Dependency>) -> Package {
    Package {
        name: name.into(),
        edition: "2024".into(),
        directory: name.into(),
        dependencies,
        targets: Vec::new(),
    }
}

fn features(source: &str, dependencies: &[Dependency]) -> PackageFeatureSet {
    let value = source.parse::<Value>().expect("parse manifest");
    PackageFeatureSet::parse(&value, dependencies).expect("parse features")
}
