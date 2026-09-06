//! The frozen whole-lock guard and its helpers execute unchanged in trusted qualification.

use serde::Deserialize;
use std::{fs, path::Path};

const CRATES_IO_SOURCE: &str = "registry+https://github.com/rust-lang/crates.io-index";

#[derive(Debug, Deserialize)]
pub(crate) struct Guardrails {
    schema: u64,
    dependencies: Dependencies,
}

#[derive(Debug, Deserialize)]
struct Dependencies {
    kafka_wire_version: String,
    kafka_wire_checksum: String,
    kafka_wire_core_version: String,
    kafka_wire_core_checksum: String,
    bornera_version: String,
    bornera_checksum: String,
    bornera_core_version: String,
    bornera_core_checksum: String,
    bornera_rustls_version: String,
    bornera_rustls_checksum: String,
}

pub(super) fn check(root: &Path) {
    let workspace_root = || root.to_path_buf();
    let root = workspace_root();
    let guardrails = load_guardrails(&root);
    let lock = parse(&root.join("Cargo.lock"));

    assert_locked(
        &lock,
        "kafka-wire",
        &guardrails.dependencies.kafka_wire_version,
        &guardrails.dependencies.kafka_wire_checksum,
    );
    assert_locked(
        &lock,
        "kafka-wire-core",
        &guardrails.dependencies.kafka_wire_core_version,
        &guardrails.dependencies.kafka_wire_core_checksum,
    );
    assert_locked(
        &lock,
        "bornera",
        &guardrails.dependencies.bornera_version,
        &guardrails.dependencies.bornera_checksum,
    );
    assert_locked(
        &lock,
        "bornera-core",
        &guardrails.dependencies.bornera_core_version,
        &guardrails.dependencies.bornera_core_checksum,
    );
    assert_locked(
        &lock,
        "bornera-rustls",
        &guardrails.dependencies.bornera_rustls_version,
        &guardrails.dependencies.bornera_rustls_checksum,
    );
}

fn assert_locked(lock: &toml::Value, name: &str, version: &str, checksum: &str) {
    let version = version
        .strip_prefix('=')
        .unwrap_or_else(|| panic!("{name} guardrail version must be exact"));
    let packages = lock["package"]
        .as_array()
        .unwrap_or_else(|| panic!("Cargo.lock must contain package entries"));
    let matches = packages
        .iter()
        .filter(|package| package["name"].as_str() == Some(name))
        .collect::<Vec<_>>();

    assert_eq!(matches.len(), 1, "{name} must resolve exactly once");
    let package = matches[0];
    assert_eq!(package["version"].as_str(), Some(version));
    assert_eq!(package["source"].as_str(), Some(CRATES_IO_SOURCE));
    assert_eq!(package["checksum"].as_str(), Some(checksum));
}

fn parse(path: &Path) -> toml::Value {
    toml::from_str(&read(path)).unwrap_or_else(|error| panic!("parse {}: {error}", path.display()))
}

pub(crate) fn load_guardrails(root: &Path) -> Guardrails {
    let source = read(&root.join("guardrails.toml"));
    let config = toml::from_str::<Guardrails>(&source)
        .unwrap_or_else(|error| panic!("parse guardrails.toml: {error}"));
    assert_eq!(config.schema, 1, "unsupported guardrails.toml schema");
    config
}

pub(crate) fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}
