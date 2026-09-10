//! Native contracts store literal identities; Cargo parses their observed versions separately.

use super::model;
use crate::cargo::ResolvedCargoGraph;
use serde::Serialize;
use std::{fs, path::Path};
use zrail_core::{LockPackageRule, load_contract, sha256_hex};

#[derive(Serialize)]
pub(super) struct Native {
    pub(super) contract_error: Option<String>,
    pub(super) graph_error: Option<String>,
    contract_sha256: String,
    contract_source: String,
    lock_sha256: String,
    lock_source: String,
    pub(super) observation: Option<crate::GovernedLockPackage>,
}

pub(super) fn contract(root: &Path, rule: &LockPackageRule) -> (String, Result<(), String>) {
    let base = fs::read_to_string(
        model::project().join("crates/zrail-testkit/tests/fixtures/good/zrail.toml"),
    )
    .expect("base contract");
    let mut value: toml::Value = base.parse().expect("base TOML");
    value
        .get_mut("dependencies")
        .expect("dependencies")
        .as_table_mut()
        .expect("table")
        .insert(
            "lock_package".into(),
            toml::Value::Array(vec![toml::Value::try_from(rule).expect("typed rule")]),
        );
    let source = toml::to_string(&value).expect("complete nested contract");
    fs::write(root.join("zrail.toml"), &source).expect("test-owned contract");
    let result = load_contract(root, Path::new("zrail.toml"))
        .map(|bundle| {
            assert_eq!(
                bundle.contract.dependencies.lock_packages.as_slice(),
                std::slice::from_ref(rule)
            );
        })
        .map_err(|error| error.to_string());
    (source, result)
}

pub(super) fn paired(root: &Path, rule: &LockPackageRule) -> (toml::Value, Native) {
    let identity = model::identity(rule);
    let mut package = toml::map::Map::new();
    for (key, value) in [
        ("name", &rule.package),
        ("version", &identity.version),
        ("source", &identity.source),
        ("checksum", identity.checksum.as_ref().expect("checksum")),
    ] {
        package.insert(key.into(), toml::Value::String(value.clone()));
    }
    let mut lock = toml::map::Map::new();
    lock.insert("version".into(), toml::Value::Integer(4));
    lock.insert(
        "package".into(),
        toml::Value::Array(vec![toml::Value::Table(package)]),
    );
    let lock = toml::Value::Table(lock);
    let source = toml::to_string(&lock).expect("paired lock");
    fs::write(root.join("Cargo.lock"), &source).expect("test-owned lock");
    let (contract_source, contract) = contract(root, rule);
    let graph = ResolvedCargoGraph::load(root, &[]).map_err(|error| error.to_string());
    let observation = if contract.is_ok() {
        graph.as_ref().ok().and_then(Option::as_ref).map(|graph| {
            crate::lock_packages::analyze(Some(graph), std::slice::from_ref(rule))
                .expect("exact inventory")
                .remove(0)
        })
    } else {
        None
    };
    (
        lock,
        Native {
            contract_error: contract.err(),
            graph_error: graph.err(),
            contract_sha256: sha256_hex(contract_source.as_bytes()),
            contract_source,
            lock_sha256: sha256_hex(source.as_bytes()),
            lock_source: source,
            observation,
        },
    )
}
