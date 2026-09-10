//! Frozen optional/default feature and simulator version assertions retain exact typed values.

use super::{manifest_dependencies, parse_manifest};
use std::{collections::BTreeSet, path::Path};

pub(super) fn rustls(root: &Path) {
    let workspace_root = || root.to_path_buf();
    let root = workspace_root();
    let value = parse_manifest(&root.join("Cargo.toml"));
    let rustls = &value["dependencies"]["rustls"];
    let policy = &value["workspace"]["dependencies"]["rustls"];
    let transport_feature = value["features"]["tls-rustls"]
        .as_array()
        .unwrap_or_else(|| panic!("tls-rustls must be an explicit feature"));
    let policy_features = policy["features"]
        .as_array()
        .unwrap_or_else(|| panic!("workspace rustls policy must name exact features"));

    assert_eq!(rustls["workspace"].as_bool(), Some(true));
    assert_eq!(rustls["optional"].as_bool(), Some(true));
    assert_eq!(policy["default-features"].as_bool(), Some(false));
    assert_eq!(
        policy_features,
        &[
            toml::Value::String("ring".to_owned()),
            toml::Value::String("std".to_owned()),
            toml::Value::String("tls12".to_owned()),
        ]
    );
    assert_eq!(
        transport_feature,
        &[
            toml::Value::String("dep:bornera-rustls".to_owned()),
            toml::Value::String("dep:rustls".to_owned()),
        ]
    );
}

pub(super) fn sasl(root: &Path) {
    let workspace_root = || root.to_path_buf();
    let root = workspace_root();
    let value = parse_manifest(&root.join("Cargo.toml"));
    let policy = &value["workspace"]["dependencies"]["sasl-scram"];
    let dependency = &value["dependencies"]["sasl-scram"];
    let features = policy["features"]
        .as_array()
        .unwrap_or_else(|| panic!("sasl-scram must name its complete feature contract"));

    assert_eq!(policy["version"].as_str(), Some("=0.0.1-rc.3"));
    assert_eq!(policy["default-features"].as_bool(), Some(false));
    assert_eq!(
        features,
        &[
            toml::Value::String("std".to_owned()),
            toml::Value::String("sha256".to_owned()),
            toml::Value::String("sha512".to_owned()),
            toml::Value::String("saslprep".to_owned()),
        ]
    );
    assert_eq!(dependency["workspace"].as_bool(), Some(true));
}

pub(super) fn sim(root: &Path) {
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
