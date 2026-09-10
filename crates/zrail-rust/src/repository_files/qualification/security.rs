//! Frozen raw qualification bodies run only inside trusted differential tests.

use std::path::Path;

use super::read;

const KAFKA_IMAGE: &str =
    "apache/kafka:4.3.1@sha256:77e3df9054047a88b520d0cc46e16696d3b22022e1d580aeccd2632df6532837";
const RUST_IMAGE: &str =
    "rust:1.88.0-bookworm@sha256:af306cfa71d987911a781c37b59d7d67d934f49684058f96cf72079c3626bfe0";

pub(super) fn secure_cluster_qualification_binds_each_advertised_host_to_its_identity(root: &Path) {
    let workspace_root = || root.to_path_buf();
    let root = workspace_root();
    let manifest = read(&root.join("package.json"));
    let compose = read(&root.join("smoke/kafka-secure-cluster.compose.yml"));
    let scenario = read(&root.join("smoke/real-kafka-secure-multi-broker.smoke.mjs"));
    let identities = read(&root.join("smoke/support/tls-identities.mjs"));

    assert!(manifest.contains(
        "\"smoke:real-kafka-secure-multi-broker\": \
         \"smoque run smoke/ --tag real-kafka-secure-multi-broker --ci\""
    ));
    assert!(compose.contains(RUST_IMAGE));
    for (number, broker) in [(1, "kafka-1"), (2, "kafka-2"), (3, "kafka-3")] {
        assert!(compose.contains(&format!("SSL://{broker}:9092")));
        assert!(compose.contains(&format!("KAFKA_{number}_SSL_SECRETS")));
    }
    assert!(identities.contains("DNS.1 = ${brokerName}"));
    assert!(scenario.contains("\"kafka-1:9092,kafka-2:9092\""));
    assert!(!scenario.contains("KAFKA_PROTOCOL_SOURCE"));
    assert!(!compose.contains("KAFKA_PROTOCOL_SOURCE"));
    assert!(!scenario.contains("kafka-protocol"));
    assert!(!compose.contains("kafka-protocol"));
    assert!(scenario.contains("RECOVERED TLS broker failover 1"));
    assert!(scenario.contains("PASS TLS broker failover 2"));
}

pub(super) fn qualification_containers_use_immutable_digests(root: &Path) {
    let workspace_root = || root.to_path_buf();
    let root = workspace_root();
    for path in [
        "smoke/kafka-cluster.compose.yml",
        "smoke/kafka-tls.compose.yml",
        "smoke/kafka-sasl.compose.yml",
        "smoke/kafka-secure-cluster.compose.yml",
        "smoke/kafka-security.compose.yml",
        "smoke/kafka.compose.yml",
    ] {
        let compose = read(&root.join(path));
        for image in compose
            .lines()
            .filter_map(|line| line.trim().strip_prefix("image: "))
        {
            assert!(
                image == KAFKA_IMAGE || image == RUST_IMAGE,
                "{path}: {image}"
            );
        }
    }
}
