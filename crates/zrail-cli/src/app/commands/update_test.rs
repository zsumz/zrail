//! Lock updates require explicit acceptance before recording gated architecture changes.

use std::fs;

use zrail_rust::build_lock;

use crate::app::commands::{
    git_base::{commit_all, git_available},
    update_fixture_test::{
        fixture_root, options, reset, write_fixture, write_gate_fixture, write_generated_fixture,
        write_manifest,
    },
};

use super::update;

#[test]
fn update_refuses_new_dependency_without_acceptance() {
    if !git_available() {
        return;
    }
    let root = fixture_root("dependency");
    reset(&root);
    write_fixture(&root);
    let lock_path = root.join("zrail.lock");
    build_lock(&root, std::path::Path::new("zrail.toml"))
        .expect("build initial lock")
        .write(&lock_path)
        .expect("write initial lock");
    commit_all(&root);
    let initial = fs::read_to_string(&lock_path).expect("read initial lock");
    fs::write(
        root.join("crates/fixture/Cargo.toml"),
        concat!(
            "[package]\n",
            "name = \"fixture\"\n",
            "version = \"0.0.0\"\n",
            "edition = \"2024\"\n\n",
            "[dependencies]\n",
            "serde = { package = \"serde\", version = \"1\" }\n",
        ),
    )
    .expect("add dependency");

    let mut options = options(&root);
    let refused = update(&options).expect("evaluate refused update");
    assert_eq!(refused.exit_code, 1);
    assert!(refused.text.contains("refused gated architecture changes"));
    assert_eq!(fs::read_to_string(&lock_path).expect("read lock"), initial);

    options.accept_grants = true;
    let accepted = update(&options).expect("accept update");
    assert_eq!(accepted.exit_code, 0);
    assert!(
        fs::read_to_string(&lock_path)
            .expect("read accepted lock")
            .contains("name = \"serde\"")
    );
    reset(&root);
}

#[test]
fn update_refuses_changed_generated_provenance_without_acceptance() {
    if !git_available() {
        return;
    }
    let root = fixture_root("generated");
    reset(&root);
    write_generated_fixture(&root, "one");
    let lock_path = root.join("zrail.lock");
    build_lock(&root, std::path::Path::new("zrail.toml"))
        .expect("build initial lock")
        .write(&lock_path)
        .expect("write initial lock");
    commit_all(&root);
    let initial = fs::read_to_string(&lock_path).expect("read initial lock");
    write_manifest(&root, "two");

    let mut options = options(&root);
    let refused = update(&options).expect("evaluate provenance update");

    assert_eq!(refused.exit_code, 1);
    assert!(refused.text.contains("UNKNOWN rust.generated-provenance"));
    assert_eq!(fs::read_to_string(&lock_path).expect("read lock"), initial);

    options.accept_grants = true;
    let accepted = update(&options).expect("accept provenance update");
    assert_eq!(accepted.exit_code, 0);
    assert_ne!(fs::read_to_string(&lock_path).expect("read lock"), initial);
    reset(&root);
}

#[test]
fn update_refuses_changed_gate_bytes_without_acceptance() {
    if !git_available() {
        return;
    }
    let root = fixture_root("gate");
    reset(&root);
    write_gate_fixture(&root);
    let lock_path = root.join("zrail.lock");
    build_lock(&root, std::path::Path::new("zrail.toml"))
        .expect("build initial lock")
        .write(&lock_path)
        .expect("write initial lock");
    commit_all(&root);
    let initial = fs::read_to_string(&lock_path).expect("read initial lock");
    fs::write(
        root.join("scripts/check"),
        "cargo test --workspace\ncargo fmt --check\n",
    )
    .expect("change gate");

    let mut options = options(&root);
    let refused = update(&options).expect("evaluate gate update");

    assert_eq!(refused.exit_code, 1);
    assert!(refused.text.contains("UNKNOWN qualification.gate-lock"));
    assert_eq!(fs::read_to_string(&lock_path).expect("read lock"), initial);

    options.accept_grants = true;
    let accepted = update(&options).expect("accept gate update");
    assert_eq!(accepted.exit_code, 0);
    assert!(accepted.text.contains("gates: 1"));
    reset(&root);
}
