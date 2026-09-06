//! Frozen publication, inheritance, and exact version assertions execute in trusted fixtures.

use super::{load_guardrails, parse_manifest};
use std::path::Path;

pub(super) fn wire(root: &Path) {
    let workspace_root = || root.to_path_buf();
    let root = workspace_root();
    let guardrails = load_guardrails(&root);
    let value = parse_manifest(&root.join("Cargo.toml"));

    assert_eq!(
        value["workspace"]["dependencies"]["kafka-wire"].as_str(),
        Some(guardrails.dependencies.kafka_wire_version.as_str())
    );
    assert_eq!(
        value["workspace"]["dependencies"]["kafka-wire-core"].as_str(),
        Some(guardrails.dependencies.kafka_wire_core_version.as_str())
    );
    assert_eq!(
        value["dependencies"]["kafka-wire"]["workspace"].as_bool(),
        Some(true)
    );
}

pub(super) fn local(root: &Path) {
    let workspace_root = || root.to_path_buf();
    let root = workspace_root();
    let manifest = parse_manifest(&root.join("Cargo.toml"));
    for package in [
        "kafka-driver",
        "kafka-driver-core",
        "kafka-driver-transport",
    ] {
        assert_eq!(
            manifest["workspace"]["dependencies"][package]["version"].as_str(),
            Some("0.1.0-rc.5"),
            "{package} must carry its path-compatible release version"
        );
    }
    assert_eq!(
        manifest["workspace"]["dependencies"]["kafka-wire-core"].as_str(),
        Some("=0.1.0-rc.3")
    );
    assert_eq!(
        manifest["workspace"]["dependencies"]["kafka-wire"].as_str(),
        Some("=0.1.0-rc.3")
    );
    assert_eq!(
        manifest["workspace"]["dependencies"]["kafka-driver-sim"]
            .get("version")
            .and_then(toml::Value::as_str),
        None
    );
    let probe = parse_manifest(&root.join("crates/kafka-driver-probe/Cargo.toml"));
    assert_eq!(
        probe["dependencies"]["kafka-wire"]["workspace"].as_bool(),
        Some(true)
    );
}

pub(super) fn publication(root: &Path) {
    let workspace_root = || root.to_path_buf();
    let root = workspace_root();
    for path in [
        "Cargo.toml",
        "crates/kafka-driver-core/Cargo.toml",
        "crates/kafka-driver-transport/Cargo.toml",
    ] {
        let manifest = parse_manifest(&root.join(path));
        let registries = manifest["package"]["publish"]
            .as_array()
            .unwrap_or_else(|| panic!("{path} must carry a registry allowlist"));
        assert_eq!(registries.len(), 1);
        assert_eq!(registries[0].as_str(), Some("crates-io"));
    }
    for path in [
        "crates/kafka-driver-sim/Cargo.toml",
        "crates/kafka-driver-probe/Cargo.toml",
    ] {
        let manifest = parse_manifest(&root.join(path));
        assert_eq!(manifest["package"]["publish"].as_bool(), Some(false));
    }
}
