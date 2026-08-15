//! Cargo workspace member patterns are strict, portable, and normalized.

use toml::Value;

use super::super::model::{Dependency, DependencyKind, DependencySource, Package};
use super::{
    expand_implicit_members, expand_members, resolve_workspace_dependencies, workspace_excludes,
    workspace_members,
};

#[test]
fn a_root_package_is_an_implicit_workspace_member() {
    let manifest = "[workspace]\nresolver = '2'"
        .parse::<Value>()
        .expect("parse workspace");

    assert!(workspace_members(&manifest).expect("members").is_empty());
    assert_eq!(
        expand_members(&[], &[".".into()], true).expect("expand root"),
        ["."]
    );
}

#[test]
fn standalone_packages_have_no_explicit_workspace_patterns() {
    let manifest = "[package]\nname = 'standalone'\nversion = '0.1.0'"
        .parse::<Value>()
        .expect("parse package");

    assert!(workspace_members(&manifest).expect("members").is_empty());
    assert!(workspace_excludes(&manifest).expect("excludes").is_empty());
}

#[test]
fn malformed_workspace_values_fail_closed() {
    let manifest = "workspace = 'invalid'"
        .parse::<Value>()
        .expect("parse manifest");

    assert!(workspace_members(&manifest).is_err());
}

#[test]
fn workspace_excludes_are_strict_string_arrays() {
    let manifest = r#"
        [workspace]
        members = ["crates/*"]
        exclude = ["crates/reference"]
    "#
    .parse::<Value>()
    .expect("parse workspace");

    assert_eq!(
        workspace_excludes(&manifest).expect("excludes"),
        ["crates/reference"]
    );
}

#[test]
fn workspace_patterns_are_portable_and_normalized() {
    let manifest = "[workspace]\nmembers = ['./crates/*']"
        .parse::<Value>()
        .expect("parse workspace");
    assert_eq!(workspace_members(&manifest).expect("members"), ["crates/*"]);

    let non_portable = "[workspace]\nmembers = ['crates\\\\*']"
        .parse::<Value>()
        .expect("parse workspace");
    assert!(workspace_members(&non_portable).is_err());
}

#[test]
fn in_tree_path_dependencies_are_implicit_members() {
    let packages = [
        Package {
            name: "root".into(),
            directory: ".".into(),
            dependencies: vec![path_dependency("member", "crates/member")],
            targets: Vec::new(),
        },
        Package {
            name: "member".into(),
            directory: "crates/member".into(),
            dependencies: Vec::new(),
            targets: Vec::new(),
        },
    ];

    let members =
        expand_implicit_members(vec![".".into()], &packages, &[]).expect("expand implicit members");

    assert_eq!(members, [".", "crates/member"]);
}

#[test]
fn only_exact_declared_member_paths_become_internal() {
    let mut packages = [
        Package {
            name: "root".into(),
            directory: ".".into(),
            dependencies: vec![
                path_dependency("member", "crates/member"),
                path_dependency("excluded", "crates/excluded"),
            ],
            targets: Vec::new(),
        },
        Package {
            name: "member".into(),
            directory: "crates/member".into(),
            dependencies: Vec::new(),
            targets: Vec::new(),
        },
        Package {
            name: "excluded".into(),
            directory: "crates/excluded".into(),
            dependencies: Vec::new(),
            targets: Vec::new(),
        },
    ];

    resolve_workspace_dependencies(&mut packages, &[".".into(), "crates/member".into()])
        .expect("resolve internal identity");

    assert!(matches!(
        packages[0].dependencies[0].source,
        DependencySource::WorkspaceMember { .. }
    ));
    assert!(matches!(
        packages[0].dependencies[1].source,
        DependencySource::RepositoryPath { .. }
    ));
}

fn path_dependency(name: &str, path: &str) -> Dependency {
    Dependency {
        alias: name.into(),
        name: name.into(),
        kind: DependencyKind::Normal,
        target: None,
        optional: false,
        default_features: true,
        features: Vec::new(),
        source: DependencySource::RepositoryPath {
            path: path.into(),
            requirement: None,
        },
    }
}
