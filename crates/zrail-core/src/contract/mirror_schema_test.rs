//! Test-mirror policy parsing stays strict and canonically renderable.

use std::{
    fs,
    path::Path,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::{ContractBundle, ContractError, format_contract_source, load_contract_with_entry};

const BASE: &str = r#"schema = 2
adapters = ["rust"]

[repository]
roots = ["src"]
workspace_members = "exact"
nested_git = "deny"
submodules = "deny"
symlinks = "inside"

[dependencies]
mode = "observed"
unassigned_packages = "allow"
cycles = "allow"

[source.rust]
module_docs = "allow"
facades = "allow"
tests = "sibling"

[source.rust.hygiene]
unsafe = "deny"
lint_suppressions = "deny"
"#;

const MIRROR: &str = r#"
[[source.rust.test_mirrors]]
production = "src/state.rs"
test = "src/state_test.rs"
name = "state_transitions"
receipt = "evidence/state.json"
inputs = ["Cargo.lock", "Cargo.toml"]
reason = "State transitions are exercised through the public surface."

[source.rust.test_mirrors.execution]
command = "cargo test --package state state_transitions --target x86_64-unknown-linux-gnu"
package = "state"
default_features = true
features = []
target = "x86_64-unknown-linux-gnu"
toolchain = "rustc 1.90.0 (example 2026-01-01)"
"#;

static FIXTURE_SEQUENCE: AtomicUsize = AtomicUsize::new(0);

#[test]
fn nested_execution_loads_and_formats_byte_stably() {
    let source = contract_source(MIRROR);
    let formatted = format_contract_source(&source).expect("format mirror contract");
    let loaded = load_source("round-trip-a", &formatted).expect("load formatted contract");

    assert_eq!(loaded.contract.source.rust.test_mirrors.len(), 1);
    assert_eq!(
        loaded.contract.source.rust.test_mirrors[0]
            .execution
            .package,
        "state"
    );
    assert!(formatted.contains("[source.rust.test_mirrors.execution]"));

    let serialized = format_contract_source(entry_source(&loaded))
        .expect("serialize loaded contract source again");
    let reloaded = load_source("round-trip-b", &serialized).expect("parse serialized contract");

    assert_eq!(serialized, formatted);
    assert_eq!(reloaded.contract, loaded.contract);
}

#[test]
fn unknown_mirror_and_execution_keys_fail_closed() {
    let unknown_mirror = MIRROR.replace(
        "reason = \"State transitions",
        "authority = \"unreviewed\"\nreason = \"State transitions",
    );
    let unknown_execution = MIRROR.replace(
        "command = \"cargo test",
        "sandbox = \"unreviewed\"\ncommand = \"cargo test",
    );

    assert_load_error("unknown-mirror", &unknown_mirror, "unknown field");
    assert_load_error("unknown-execution", &unknown_execution, "unknown field");
}

#[test]
fn missing_execution_field_and_member_fail_closed() {
    let missing_execution = MIRROR
        .split("[source.rust.test_mirrors.execution]")
        .next()
        .expect("mirror prefix");
    let missing_member = MIRROR.replace("toolchain = \"rustc 1.90.0 (example 2026-01-01)\"\n", "");

    assert_load_error(
        "missing-execution",
        missing_execution,
        "missing field `execution`",
    );
    assert_load_error(
        "missing-member",
        &missing_member,
        "missing field `toolchain`",
    );
}

#[test]
fn duplicate_execution_field_and_member_fail_closed() {
    let duplicate_field = MIRROR.replace(
        "[source.rust.test_mirrors.execution]",
        concat!(
            "execution = { command = \"cargo test\", package = \"state\", ",
            "default_features = true, features = [], target = \"host\", ",
            "toolchain = \"rustc 1.90.0\" }\n",
            "execution = { command = \"cargo test\", package = \"state\", ",
            "default_features = true, features = [], target = \"host\", ",
            "toolchain = \"rustc 1.90.0\" }\n",
            "[source.rust.test_mirrors.execution]",
        ),
    );
    let duplicate_member = MIRROR.replace(
        "command = \"cargo test",
        "command = \"cargo test --duplicate\"\ncommand = \"cargo test",
    );

    assert_load_error("duplicate-execution", &duplicate_field, "duplicate key");
    assert_load_error("duplicate-member", &duplicate_member, "duplicate key");
}

fn assert_load_error(label: &str, mirror: &str, expected: &str) {
    let error = load_source(label, &contract_source(mirror)).expect_err("contract must fail");
    assert!(
        error.to_string().contains(expected),
        "expected {expected:?} in {error}"
    );
}

fn contract_source(mirror: &str) -> String {
    format!("{BASE}{mirror}")
}

fn entry_source(bundle: &ContractBundle) -> &str {
    &bundle
        .sources
        .iter()
        .find(|source| source.path == "zrail.toml")
        .expect("entry source")
        .content
}

fn load_source(label: &str, source: &str) -> Result<ContractBundle, ContractError> {
    let serial = FIXTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "zrail-mirror-schema-{label}-{}-{serial}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("create fixture root");
    fs::write(root.join("zrail.toml"), BASE).expect("write fixture contract");
    let result = load_contract_with_entry(&root, Path::new("zrail.toml"), source);
    fs::remove_dir_all(root).expect("remove fixture root");
    result
}
