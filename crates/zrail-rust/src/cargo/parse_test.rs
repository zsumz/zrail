//! Cargo manifests are regular bounded files with strict package identity.

use std::fs;

use toml::Value;

use super::{package_name, read_manifest_counted};

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
