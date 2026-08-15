//! Canonical lock handling validates exact state and distinguishes absent inputs.

use std::{fs, path::PathBuf};

use super::{
    LockFile, LockedDependency, LockedDependencyKind, LockedDependencyScope, LockedGate,
    LockedGeneratedSource, LockedPackage, LockedRatchet,
};

#[test]
fn render_sorts_packages_dependencies_and_ratchets() {
    let mut lock = LockFile::new("0".repeat(64));
    lock.packages = vec![
        LockedPackage {
            name: "z".into(),
            dependencies: vec![dependency(
                "serde",
                LockedDependencyKind::Normal,
                LockedDependencyScope::External,
            )],
        },
        LockedPackage {
            name: "a".into(),
            dependencies: vec![
                dependency(
                    "z",
                    LockedDependencyKind::Development,
                    LockedDependencyScope::Internal,
                ),
                dependency(
                    "z",
                    LockedDependencyKind::Normal,
                    LockedDependencyScope::Internal,
                ),
            ],
        },
    ];
    lock.ratchets = vec![LockedRatchet {
        rule: "rust.file-size".into(),
        target: "z.rs".into(),
        value: 260,
    }];
    lock.generated = vec![LockedGeneratedSource {
        root: "src/generated".into(),
        manifest_sha256: "1".repeat(64),
    }];
    lock.gates = vec![LockedGate {
        name: "check".into(),
        path: "scripts/check".into(),
        sha256: "2".repeat(64),
    }];

    let rendered = lock.render().expect("render canonical lock");

    assert!(rendered.find("name = \"a\"") < rendered.find("name = \"z\""));
    assert!(rendered.contains("kind = \"development\""));
    assert!(rendered.contains("scope = \"internal\""));
    assert!(rendered.contains("manifest_sha256 = \"111111"));
    assert!(rendered.contains("path = \"scripts/check\""));
}

#[test]
fn duplicate_dependencies_are_rejected_instead_of_normalized_away() {
    let mut lock = LockFile::new("0".repeat(64));
    let repeated = dependency(
        "serde",
        LockedDependencyKind::Normal,
        LockedDependencyScope::External,
    );
    lock.packages.push(LockedPackage {
        name: "a".into(),
        dependencies: vec![repeated.clone(), repeated],
    });

    let error = lock.render().expect_err("duplicate dependency must fail");

    assert!(error.to_string().contains("duplicate dependency"));
}

#[test]
fn generated_manifest_digests_must_be_exact() {
    let mut lock = LockFile::new("0".repeat(64));
    lock.generated.push(LockedGeneratedSource {
        root: "src/generated".into(),
        manifest_sha256: "ABC".into(),
    });

    let error = lock
        .render()
        .expect_err("invalid provenance digest must fail");

    assert!(error.to_string().contains("invalid manifest_sha256"));
}

#[test]
fn qualification_gate_digests_must_be_exact() {
    let mut lock = LockFile::new("0".repeat(64));
    lock.gates.push(LockedGate {
        name: "check".into(),
        path: "scripts/check".into(),
        sha256: "ABC".into(),
    });

    let error = lock.render().expect_err("invalid gate digest must fail");

    assert!(error.to_string().contains("invalid sha256"));
}

#[test]
fn optional_read_accepts_only_a_genuinely_absent_lock() {
    let root = fixture_root("optional");
    reset(&root);
    let path = root.join("zrail.lock");

    assert_eq!(LockFile::read_optional(&path).expect("read absence"), None);
    fs::create_dir(&path).expect("create non-file lock");
    let error = LockFile::read_optional(&path).expect_err("directory is not absence");
    assert!(error.to_string().contains("regular file"));
    fs::remove_dir_all(root).expect("remove fixture");
}

#[cfg(unix)]
#[test]
fn optional_read_rejects_a_dangling_lock_alias() {
    use std::os::unix::fs::symlink;

    let root = fixture_root("dangling");
    reset(&root);
    let path = root.join("zrail.lock");
    symlink(root.join("missing"), &path).expect("create dangling alias");

    let error = LockFile::read_optional(&path).expect_err("alias is not absence");

    assert!(error.to_string().contains("symlink"));
    fs::remove_dir_all(root).expect("remove fixture");
}

fn dependency(
    name: &str,
    kind: LockedDependencyKind,
    scope: LockedDependencyScope,
) -> LockedDependency {
    LockedDependency {
        name: name.into(),
        kind,
        scope,
    }
}

fn fixture_root(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("zrail-lock-{name}-{}", std::process::id()))
}

fn reset(root: &PathBuf) {
    if root.exists() {
        fs::remove_dir_all(root).expect("reset fixture");
    }
    fs::create_dir_all(root).expect("create fixture");
}
