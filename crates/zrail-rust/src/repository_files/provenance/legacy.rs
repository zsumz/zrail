//! Frozen provenance assertion body and helpers execute only in trusted qualification.

use serde::Deserialize;
use std::{fs, path::Path};
use zrail_core::RepositoryFileRule;

#[derive(Debug, Deserialize)]
pub(crate) struct Guardrails {
    schema: u64,
    dependencies: Dependencies,
}

#[derive(Debug, Deserialize)]
#[expect(
    clippy::struct_field_names,
    reason = "The byte-exact frozen guard reads these reviewed registry keys."
)]
struct Dependencies {
    kafka_wire_version: String,
    kafka_wire_core_version: String,
    bornera_version: String,
    bornera_core_version: String,
    bornera_rustls_version: String,
}

pub(super) fn check(root: &Path, policy: &RepositoryFileRule) {
    if policy.name.starts_with("kd-provenance-00-parse-") {
        parse(&root.join(&policy.include[0]));
    } else {
        manifests(root);
    }
}

fn manifests(root: &Path) {
    let workspace_root = || root.to_path_buf();
    let root = workspace_root();
    let guardrails = load_guardrails(&root);
    let workspace = parse(&root.join("Cargo.toml"));
    let probe = parse(&root.join("crates/kafka-driver-probe/Cargo.toml"));

    for (package, expected) in [
        (
            "kafka-wire",
            guardrails.dependencies.kafka_wire_version.as_str(),
        ),
        (
            "kafka-wire-core",
            guardrails.dependencies.kafka_wire_core_version.as_str(),
        ),
        ("bornera", guardrails.dependencies.bornera_version.as_str()),
        (
            "bornera-core",
            guardrails.dependencies.bornera_core_version.as_str(),
        ),
        (
            "bornera-rustls",
            guardrails.dependencies.bornera_rustls_version.as_str(),
        ),
    ] {
        let dependency = &workspace["workspace"]["dependencies"][package];
        assert!(
            dependency.is_str(),
            "{package} must be an exact registry version string"
        );
        assert_eq!(
            dependency.as_str(),
            Some(expected),
            "{package} must be exact"
        );
    }
    assert_workspace_reference(&workspace["dependencies"]["kafka-wire"]);
    assert_workspace_reference(&workspace["dependencies"]["kafka-wire-core"]);
    assert_workspace_reference(&probe["dependencies"]["kafka-wire"]);
    assert_workspace_reference(&workspace["dependencies"]["bornera"]);
    assert_workspace_reference(&workspace["dependencies"]["bornera-core"]);
    assert_optional_workspace_reference(&workspace["dependencies"]["bornera-rustls"]);

    for manifest in [&workspace, &probe] {
        assert!(
            manifest.get("patch").is_none(),
            "registry patches are forbidden"
        );
        assert!(
            manifest.get("replace").is_none(),
            "dependency replacement is forbidden"
        );
        assert!(
            manifest.get("target").is_none(),
            "target-specific protocol redeclaration is forbidden"
        );
    }
}

fn assert_optional_workspace_reference(dependency: &toml::Value) {
    let table = dependency
        .as_table()
        .unwrap_or_else(|| panic!("optional dependency must use the workspace authority"));
    assert_eq!(
        table.len(),
        2,
        "optional dependency may only add its optional marker"
    );
    assert_eq!(
        table.get("workspace").and_then(toml::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        table.get("optional").and_then(toml::Value::as_bool),
        Some(true)
    );
}

fn assert_workspace_reference(dependency: &toml::Value) {
    let table = dependency
        .as_table()
        .unwrap_or_else(|| panic!("protocol consumer must use the workspace dependency"));
    assert_eq!(
        table.len(),
        1,
        "protocol dependency may not add source aliases"
    );
    assert_eq!(
        table.get("workspace").and_then(toml::Value::as_bool),
        Some(true)
    );
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
