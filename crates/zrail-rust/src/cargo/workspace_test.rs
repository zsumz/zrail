//! Cargo workspace member patterns are strict, portable, and normalized.

use toml::Value;

use super::super::model::{DependencyPath, Package};
use super::{expand_implicit_members, expand_members, workspace_excludes, workspace_members};

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
            dependencies: Vec::new(),
            dependency_paths: vec![DependencyPath {
                path: "crates/member".into(),
                workspace_relative: false,
            }],
            targets: Vec::new(),
        },
        Package {
            name: "member".into(),
            directory: "crates/member".into(),
            dependencies: Vec::new(),
            dependency_paths: Vec::new(),
            targets: Vec::new(),
        },
    ];

    let members =
        expand_implicit_members(vec![".".into()], &packages, &[]).expect("expand implicit members");

    assert_eq!(members, [".", "crates/member"]);
}
