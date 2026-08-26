//! Filesystem setup for governed-surface report test repositories.

use std::{fs, path::PathBuf};

use super::{CONTRACT, LIBRARY, LOCK, MANIFEST, MIRROR, OWNER};

pub(in super::super) fn repository(label: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "zrail-coverage-{label}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create source");
    fs::create_dir_all(root.join("tests")).expect("create tests");
    fs::create_dir_all(root.join("evidence")).expect("create evidence");
    fs::create_dir_all(root.join("artifacts/owned")).expect("create owned directory");
    fs::create_dir_all(root.join("artifacts/trespass")).expect("create trespass directory");
    write(&root, "Cargo.toml", MANIFEST);
    write(&root, "Cargo.lock", LOCK);
    write(&root, "zrail.toml", CONTRACT);
    write(&root, "src/lib.rs", LIBRARY);
    write(&root, "src/owner.rs", OWNER);
    write(&root, "tests/mirror.rs", MIRROR);
    write(&root, "evidence/mirror.json", "{}\n");
    root
}

pub(in super::super) fn write(root: &std::path::Path, path: &str, contents: &str) {
    fs::write(root.join(path), contents).expect("write fixture");
}

pub(in super::super) fn reset(root: &std::path::Path) {
    if root.exists() {
        fs::remove_dir_all(root).expect("reset fixture");
    }
}
