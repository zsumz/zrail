//! Cargo feature closure retains exact local and optional-dependency semantics.

use toml::Value;

use super::PackageFeatureSet;
use crate::cargo::{CrateRootAuthority, Dependency, DependencyKind, DependencySource};

#[test]
fn resolves_default_local_cycles_and_implicit_optional_features() {
    let manifest = r#"
[features]
default = ["strict"]
strict = ["trace"]
trace = ["strict"]
local-runtime = []
"#
    .parse::<Value>()
    .expect("parse manifest");
    let features =
        PackageFeatureSet::parse(&manifest, &[optional("local-runtime")]).expect("parse features");

    assert_eq!(
        features
            .resolve(true, &[])
            .expect("resolve defaults")
            .into_iter()
            .collect::<Vec<_>>(),
        ["default", "strict", "trace"]
    );
    assert!(features.declared().contains("local-runtime"));
}

#[test]
fn explicit_dependency_activation_suppresses_implicit_feature_and_rejects_unknowns() {
    let manifest = r#"
[features]
runtime = ["dep:local-runtime", "local-runtime/io"]
"#
    .parse::<Value>()
    .expect("parse manifest");
    let features = PackageFeatureSet::parse(&manifest, &[optional("local-runtime")])
        .expect("parse dependency activation");

    assert!(!features.declared().contains("local-runtime"));
    assert!(features.resolve(false, &["missing".into()]).is_err());
    let resolved = features
        .resolve_details(false, &["runtime".into()])
        .expect("resolve dependency activation");
    assert!(resolved.enabled_dependencies.contains("local-runtime"));
    assert_eq!(
        resolved.dependency_features["local-runtime"]
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["io"]
    );

    let invalid = "[features]\nbroken = ['missing']\n"
        .parse::<Value>()
        .expect("parse invalid feature");
    assert!(PackageFeatureSet::parse(&invalid, &[]).is_err());

    let weak = "[features]\nbroken = ['required?/feature']\n"
        .parse::<Value>()
        .expect("parse weak activation");
    let mut required = optional("required");
    required.optional = false;
    assert!(PackageFeatureSet::parse(&weak, &[required]).is_err());
}

#[test]
fn weak_dependency_features_do_not_enable_optional_dependencies() {
    let manifest = "[features]\ninspect = ['local-runtime?/trace']\n"
        .parse::<Value>()
        .expect("parse weak feature");
    let features = PackageFeatureSet::parse(&manifest, &[optional("local-runtime")])
        .expect("parse weak activation");

    let resolved = features
        .resolve_details(false, &["inspect".into()])
        .expect("resolve weak activation");

    assert!(!resolved.enabled_dependencies.contains("local-runtime"));
    assert!(resolved.dependency_features["local-runtime"].contains("trace"));
}

#[test]
fn accepts_cargo_unicode_xid_feature_names() {
    let manifest = "[features]\n\"δelta\" = []\n"
        .parse::<Value>()
        .expect("parse Unicode feature");
    let features = PackageFeatureSet::parse(&manifest, &[]).expect("accept Cargo feature name");

    assert!(features.resolve(false, &["δelta".into()]).is_ok());
}

fn optional(alias: &str) -> Dependency {
    Dependency {
        alias: alias.into(),
        name: alias.into(),
        explicit_package: false,
        crate_root: alias.replace('-', "_"),
        crate_root_authority: CrateRootAuthority::DeclaredAlias,
        kind: DependencyKind::Normal,
        target: None,
        optional: true,
        default_features: true,
        features: Vec::new(),
        source: DependencySource::Registry {
            registry: None,
            index: None,
            requirement: "1".into(),
        },
    }
}
