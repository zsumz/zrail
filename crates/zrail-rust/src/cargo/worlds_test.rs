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
    assert!(error.contains("package \"core\" feature \"trace\""));
    assert!(error.contains("normal dependency edge from package \"app\""));
    assert!(error.contains("alias \"core\""));
    assert!(error.contains("target condition \"cfg(unix)\""));
    assert!(error.contains("every relevant Cargo compilation context to converge"));
}

#[test]
fn rejects_build_and_development_feature_splits_with_exact_edge_diagnostics() {
    for (kind, label) in [
        (DependencyKind::Build, "build"),
        (DependencyKind::Development, "development"),
    ] {
        let mut cargo = workspace(None);
        cargo.packages[0].dependencies[0].kind = kind;

        let error = resolve_feature_worlds(&cargo, &[world()]).expect_err("reject context split");

        assert!(error.contains(&format!("{label} dependency edge")));
        assert!(error.contains("package \"core\" feature \"trace\""));
        assert!(error.contains("target condition <all targets>"));
    }
}

#[test]
fn accepts_target_conditional_edges_when_split_package_features_are_empty() {
    let spec = FeatureWorldSpec {
        name: "empty".into(),
        packages: ["app", "core"]
            .into_iter()
            .map(|package| FeaturePackageSelection {
                package: package.into(),
                default_features: false,
                features: Vec::new(),
            })
            .collect(),
    };

    let worlds = resolve_feature_worlds(&workspace(Some("cfg(unix)")), &[spec])
        .expect("feature-empty split package is exact");

    assert!(worlds[0].packages["app"].active.is_empty());
    assert!(worlds[0].packages["core"].active.is_empty());
}

#[test]
fn inactive_optional_split_edges_taint_downstream_packages_structurally() {
    let split = Dependency {
        alias: "helper".into(),
        name: "helper".into(),
        explicit_package: false,
        crate_root: "helper".into(),
        crate_root_authority: CrateRootAuthority::DeclaredAlias,
        kind: DependencyKind::Normal,
        target: Some("cfg(unix)".into()),
        optional: true,
        default_features: false,
        features: Vec::new(),
        source: DependencySource::WorkspaceMember {
            directory: "helper".into(),
            requirement: None,
        },
    };
    let downstream = Dependency {
        alias: "core".into(),
        name: "core".into(),
        explicit_package: false,
        crate_root: "core".into(),
        crate_root_authority: CrateRootAuthority::DeclaredAlias,
        kind: DependencyKind::Normal,
        target: None,
        optional: false,
        default_features: false,
        features: vec!["trace".into()],
        source: DependencySource::WorkspaceMember {
            directory: "core".into(),
            requirement: None,
        },
    };
    let cargo = CargoWorkspace {
        declared_members: vec!["app".into(), "core".into(), "helper".into()],
        observed_members: vec!["app".into(), "core".into(), "helper".into()],
        packages: vec![
            package("app", vec![split.clone()]),
            package("core", Vec::new()),
            package("helper", vec![downstream.clone()]),
        ],
        package_features: BTreeMap::from([
            ("app".into(), features("[features]\n", &[split])),
            ("core".into(), features("[features]\ntrace = []\n", &[])),
            ("helper".into(), features("[features]\n", &[downstream])),
        ]),
        authority_surfaces: Vec::new(),
        manifest_scopes: BTreeMap::new(),
    };
    let spec = FeatureWorldSpec {
        name: "empty".into(),
        packages: ["app", "core", "helper"]
            .into_iter()
            .map(|package| FeaturePackageSelection {
                package: package.into(),
                default_features: false,
                features: Vec::new(),
            })
            .collect(),
    };

    let error = resolve_feature_worlds(&cargo, &[spec]).expect_err("reject structural split taint");

    assert!(error.contains("package \"core\" feature \"trace\""));
    assert!(error.contains("package \"helper\" is context-split"));
    assert!(error.contains("target condition \"cfg(unix)\""));
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
