//! Frozen raw qualification bodies run only inside trusted differential tests.

use std::path::Path;

use super::read;

pub(super) fn canonical_gate_uses_the_locked_graph_and_detects_mutation(root: &Path) {
    let workspace_root = || root.to_path_buf();
    let root = workspace_root();
    let gate = read(&root.join("scripts/check"));

    assert_eq!(gate.matches("--locked").count(), 4);
    assert!(gate.contains("git diff --check"));
    assert!(gate.contains("git diff --exit-code"));
}

pub(super) fn ci_prefetches_before_running_the_gate_offline(root: &Path) {
    let workspace_root = || root.to_path_buf();
    let root = workspace_root();
    let workflow = read(&root.join(".github/workflows/ci.yml"));

    assert!(workflow.contains("run: cargo fetch --locked"));
    assert!(workflow.contains("CARGO_NET_OFFLINE: \"true\""));
    assert!(workflow.contains("run: scripts/check"));
}

pub(super) fn functional_smoke_runs_on_pull_requests_and_main_without_performance(root: &Path) {
    let workspace_root = || root.to_path_buf();
    let root = workspace_root();
    let workflow = read(&root.join(".github/workflows/smoke.yml"));
    let manifest = read(&root.join("package.json"));

    assert!(workflow.contains("pull_request:"));
    assert!(workflow.contains("branches: [main]"));
    assert!(workflow.contains("contents: read"));
    assert!(!workflow.contains("kafka-protocol"));
    assert!(!workflow.contains("repository: kafkars/kafka-wire"));
    assert!(workflow.contains("run: npm run qualify:real-kafka-functional"));
    assert!(!workflow.contains("run: npm run qualify:real-kafka\n"));
    assert!(!workflow.contains("run: npm run measure:real-kafka"));
    assert!(manifest.contains(
        "\"qualify:real-kafka-functional\": \"npm run test:qualification-policy && \
         smoque run smoke/ --tag real-kafka-functional --ci\""
    ));

    for path in [
        "smoke/real-kafka.smoke.mjs",
        "smoke/real-kafka-movement.smoke.mjs",
        "smoke/real-kafka-multi-broker.smoke.mjs",
        "smoke/real-kafka-reconnect.smoke.mjs",
        "smoke/real-kafka-sasl.smoke.mjs",
        "smoke/real-kafka-secure-multi-broker.smoke.mjs",
        "smoke/real-kafka-security.smoke.mjs",
        "smoke/real-kafka-tls.smoke.mjs",
    ] {
        assert!(read(&root.join(path)).contains("real-kafka-functional"));
    }
    assert!(
        !read(&root.join("smoke/real-kafka-performance.smoke.mjs"))
            .contains("real-kafka-functional")
    );
}

pub(super) fn ci_uses_the_locked_registry_protocol_without_a_sibling_checkout(root: &Path) {
    let workspace_root = || root.to_path_buf();
    let root = workspace_root();
    let workflow = read(&root.join(".github/workflows/ci.yml"));

    assert!(workflow.contains("run: cargo fetch --locked"));
    assert!(!workflow.contains("kafka-protocol"));
    assert!(!workflow.contains("repository: kafkars/kafka-wire"));
    assert!(!workflow.contains("KAFKA_PROTOCOL_SSH_KEY"));
    assert!(!workflow.contains("KAFKA_PROTOCOL_TOKEN"));
}
