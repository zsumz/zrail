//! Frozen assertion dispatch is trusted fixture code; runtime analysis never calls it.

#[path = "legacy/features.rs"]
mod features;
#[path = "legacy/releases.rs"]
mod releases;

use serde::Deserialize;
use std::{collections::BTreeSet, fs, path::Path};
use zrail_core::RepositoryFileRule;

#[derive(Deserialize)]
pub(crate) struct Guardrails {
    schema: u32,
    dependencies: Dependencies,
}
#[derive(Deserialize)]
struct Dependencies {
    kafka_wire_version: String,
    kafka_wire_core_version: String,
}

pub(super) fn check(root: &Path, rule: &RepositoryFileRule) {
    if rule.name.starts_with("kd-dep-00-parse-") {
        let _ = parse_manifest(&root.join(&rule.include[0]));
        return;
    }
    match rule.name.as_str() {
        "kd-dep-driver-wire-inheritance"
        | "kd-dep-kafka-wire-core-registry-version"
        | "kd-dep-kafka-wire-registry-version" => releases::wire(root),
        "kd-dep-kafka-driver-core-path-release"
        | "kd-dep-kafka-driver-path-release"
        | "kd-dep-kafka-driver-transport-path-release"
        | "kd-dep-kafka-wire-core-release"
        | "kd-dep-kafka-wire-release"
        | "kd-dep-probe-wire-inheritance"
        | "kd-dep-sim-no-version-string" => releases::local(root),
        "kd-dep-kafka-driver-core-public-registry"
        | "kd-dep-kafka-driver-probe-private"
        | "kd-dep-kafka-driver-public-registry"
        | "kd-dep-kafka-driver-sim-private"
        | "kd-dep-kafka-driver-transport-public-registry" => releases::publication(root),
        "kd-dep-rustls-features"
        | "kd-dep-rustls-inheritance"
        | "kd-dep-rustls-no-defaults"
        | "kd-dep-rustls-optional"
        | "kd-dep-tls-features" => features::rustls(root),
        "kd-dep-sasl-features"
        | "kd-dep-sasl-inheritance"
        | "kd-dep-sasl-no-defaults"
        | "kd-dep-sasl-version" => features::sasl(root),
        "kd-dep-criticality-version" => features::sim(root),
        _ => panic!("unknown frozen dependency field"),
    }
}

fn manifest_dependencies(path: &std::path::Path) -> BTreeSet<String> {
    let value = parse_manifest(path);

    value["dependencies"]
        .as_table()
        .map_or_else(BTreeSet::new, |dependencies| {
            dependencies.keys().cloned().collect()
        })
}

fn parse_manifest(path: &std::path::Path) -> toml::Value {
    read(path)
        .parse::<toml::Value>()
        .unwrap_or_else(|error| panic!("parse {}: {error}", path.display()))
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
