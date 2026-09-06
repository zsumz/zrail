//! Frozen dependency assertion bodies and manifest readers; trusted qualification only.

use serde::Deserialize;
use std::{collections::BTreeSet, fs, path::Path};
use zrail_core::RepositoryFileRule;

#[derive(Deserialize)]
pub(crate) struct Guardrails {
    schema: u32,
    dependencies: Dependencies,
}
#[derive(Deserialize)]
#[expect(
    clippy::struct_field_names,
    reason = "Preserve the frozen registry field names used by the unchanged assertion bodies."
)]
struct Dependencies {
    driver_allowed: Vec<String>,
    core_allowed: Vec<String>,
    transport_allowed: Vec<String>,
    probe_allowed: Vec<String>,
}

pub(super) fn check(root: &Path, rule: &RepositoryFileRule) {
    match rule.name.as_str() {
        "kd-dep-driver-keys" => driver(root),
        "kd-dep-core-keys" => core(root),
        "kd-dep-transport-keys" => transport(root),
        "kd-dep-sim-keys" => sim(root),
        "kd-dep-probe-keys" => probe(root),
        _ => panic!("unknown frozen key policy"),
    }
}

fn driver(root: &Path) {
    let workspace_root = || root.to_path_buf();
    let root = workspace_root();
    let guardrails = load_guardrails(&root);
    let dependencies = manifest_dependencies(&root.join("Cargo.toml"));
    let allowed = guardrails
        .dependencies
        .driver_allowed
        .into_iter()
        .collect::<BTreeSet<_>>();

    assert_eq!(dependencies, allowed);
}

fn core(root: &Path) {
    let workspace_root = || root.to_path_buf();
    let root = workspace_root();
    let guardrails = load_guardrails(&root);
    let dependencies = manifest_dependencies(&root.join("crates/kafka-driver-core/Cargo.toml"));
    let allowed = guardrails
        .dependencies
        .core_allowed
        .into_iter()
        .collect::<BTreeSet<_>>();
    let violations = dependencies.difference(&allowed).collect::<Vec<_>>();

    assert!(
        violations.is_empty(),
        "the deterministic core acquired forbidden dependencies: {violations:?}"
    );
}

fn transport(root: &Path) {
    let workspace_root = || root.to_path_buf();
    let root = workspace_root();
    let guardrails = load_guardrails(&root);
    let dependencies =
        manifest_dependencies(&root.join("crates/kafka-driver-transport/Cargo.toml"));
    let allowed = guardrails
        .dependencies
        .transport_allowed
        .into_iter()
        .collect::<BTreeSet<_>>();

    assert_eq!(dependencies, allowed);
}

fn sim(root: &Path) {
    let workspace_root = || root.to_path_buf();
    let root = workspace_root();
    let dependencies = manifest_dependencies(&root.join("crates/kafka-driver-sim/Cargo.toml"));

    assert_eq!(
        dependencies,
        BTreeSet::from(["criticality".to_owned(), "kafka-driver-core".to_owned()])
    );
    let workspace = parse_manifest(&root.join("Cargo.toml"));
    assert_eq!(
        workspace["workspace"]["dependencies"]["criticality"].as_str(),
        Some("=0.0.1-rc.2")
    );
}

fn probe(root: &Path) {
    let workspace_root = || root.to_path_buf();
    let root = workspace_root();
    let guardrails = load_guardrails(&root);
    let dependencies = manifest_dependencies(&root.join("crates/kafka-driver-probe/Cargo.toml"));
    let allowed = guardrails
        .dependencies
        .probe_allowed
        .into_iter()
        .collect::<BTreeSet<_>>();

    assert_eq!(dependencies, allowed);
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
