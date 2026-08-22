//! Cargo manifests are regular bounded files with strict package identity.

use std::fs;

use toml::Value;

use crate::inventory::inventory_cargo_repository;

use super::{load_cargo_workspace, package_name, read_manifest_counted};

#[test]
fn malformed_package_identity_is_not_a_virtual_manifest() {
    let manifest = "package = 'wrong'"
        .parse::<Value>()
        .expect("parse manifest");

    assert!(package_name(&manifest).is_err());
}

#[cfg(unix)]
#[test]
fn cargo_manifest_symlinks_are_rejected_before_reading() {
    use std::os::unix::fs::symlink;

    let root = std::env::temp_dir().join(format!("zrail-cargo-alias-{}", std::process::id()));
    if root.exists() {
        fs::remove_dir_all(&root).expect("reset fixture");
    }
    fs::create_dir_all(&root).expect("create fixture");
    fs::write(root.join("actual.toml"), "[workspace]\n").expect("write target");
    symlink(root.join("actual.toml"), root.join("Cargo.toml")).expect("create alias");

    let error =
        read_manifest_counted(&root.join("Cargo.toml"), &mut 0).expect_err("alias must fail");

    assert!(error.to_string().contains("symlink"));
    fs::remove_dir_all(root).expect("remove fixture");
}

#[test]
fn path_dependencies_become_implicit_active_members() {
    let root = fixture_root("implicit-member");
    reset(&root);
    write(
        &root.join("Cargo.toml"),
        "[workspace]\nmembers = []\n\n[package]\nname = 'root'\nversion = '0.0.0'\n\n[dependencies]\nmember = { path = 'crates/member' }\n",
    );
    write(
        &root.join("crates/member/Cargo.toml"),
        "[package]\nname = 'member'\nversion = '0.0.0'\n",
    );
    write(&root.join("src/lib.rs"), "//! root\n");
    write(&root.join("crates/member/src/lib.rs"), "//! member\n");

    let workspace = load(&root).expect("load active workspace");

    assert_eq!(workspace.declared_members, [".", "crates/member"]);
    assert_eq!(workspace.packages.len(), 2);
    reset(&root);
}

#[test]
fn unrelated_nested_workspace_is_not_resolved_against_the_root() {
    let root = fixture_root("unrelated-workspace");
    reset(&root);
    write(
        &root.join("Cargo.toml"),
        "[workspace]\nmembers = ['crates/app']\nexclude = ['reference/*']\n",
    );
    write(
        &root.join("crates/app/Cargo.toml"),
        "[package]\nname = 'app'\nversion = '0.0.0'\n",
    );
    write(&root.join("crates/app/src/lib.rs"), "//! app\n");
    write(
        &root.join("reference/example/Cargo.toml"),
        "[workspace]\nmembers = ['member']\n[workspace.dependencies]\nlocal = { path = 'shared' }\n",
    );
    write(
        &root.join("reference/example/member/Cargo.toml"),
        "[package]\nname = 'example'\nversion = '0.0.0'\n[dependencies]\nlocal.workspace = true\n",
    );

    let workspace = load(&root).expect("ignore nested workspace");

    assert_eq!(workspace.observed_members, ["crates/app"]);
    assert_eq!(workspace.packages[0].name, "app");
    assert!(workspace.source_is_active("crates/app/src/lib.rs"));
    assert!(!workspace.source_is_active("reference/example/member/src/lib.rs"));
    reset(&root);
}

#[test]
fn active_path_dependency_cannot_cross_a_nested_workspace() {
    let root = fixture_root("workspace-crossing");
    reset(&root);
    write(
        &root.join("Cargo.toml"),
        "[package]\nname = 'root'\nversion = '0.0.0'\n[dependencies]\nnested = { path = 'nested/member' }\n",
    );
    write(&root.join("src/lib.rs"), "//! root\n");
    write(
        &root.join("nested/Cargo.toml"),
        "[workspace]\nmembers = ['member']\n",
    );
    write(
        &root.join("nested/member/Cargo.toml"),
        "[package]\nname = 'nested'\nversion = '0.0.0'\n",
    );

    let error = load(&root).expect_err("nested workspace crossing must fail");

    assert!(error.to_string().contains(
        "path dependency from \".\" crosses from workspace \".\" into nested workspace \"nested\""
    ));
    reset(&root);
}

fn load(root: &std::path::Path) -> Result<super::CargoWorkspace, super::CargoModelError> {
    let inventory = inventory_cargo_repository(root).expect("inventory Cargo repository");
    load_cargo_workspace(&inventory)
}

fn write(path: &std::path::Path, contents: &str) {
    fs::create_dir_all(path.parent().expect("fixture path has parent"))
        .expect("create fixture parent");
    fs::write(path, contents).expect("write fixture");
}

fn fixture_root(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("zrail-cargo-{name}-{}", std::process::id()))
}

fn reset(root: &std::path::Path) {
    if root.exists() {
        fs::remove_dir_all(root).expect("reset fixture");
    }
}
