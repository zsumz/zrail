//! Frozen raw qualification bodies run only inside trusted differential tests.

use std::path::Path;

use super::read;

pub(super) fn release_qualification_runs_the_canonical_gate_before_packaging(root: &Path) {
    let workspace_root = || root.to_path_buf();
    let root = workspace_root();
    let workflow = read(&root.join(".github/workflows/qualification.yml"));
    let fetch = workflow
        .find("run: cargo fetch --locked")
        .unwrap_or_else(|| panic!("release qualification must fetch the locked graph"));
    let gate = workflow
        .find("run: scripts/check")
        .unwrap_or_else(|| panic!("release qualification must run the canonical gate"));
    let packages = workflow
        .find("run: scripts/qualify-packages")
        .unwrap_or_else(|| panic!("release qualification must qualify package archives"));

    assert!(workflow.contains("tags: [\"v*\"]"));
    assert!(
        workflow.contains(
            "rustup toolchain install 1.88.0 --profile minimal --component clippy,rustfmt"
        )
    );
    assert!(fetch < gate);
    assert!(gate < packages);
    assert!(workflow[gate..packages].contains("CARGO_NET_OFFLINE: \"true\""));
}

pub(super) fn release_qualification_builds_normalized_public_archives(root: &Path) {
    let workspace_root = || root.to_path_buf();
    let root = workspace_root();
    let workflow = read(&root.join(".github/workflows/qualification.yml"));
    let prefetch = read(&root.join("scripts/prefetch-release-dependencies"));
    let script = read(&root.join("scripts/qualify-packages"));

    assert!(workflow.contains("run: scripts/prefetch-release-dependencies"));
    assert!(workflow.contains("run: scripts/qualify-packages"));
    assert!(workflow.contains("run: cargo fetch --locked"));
    assert!(workflow.contains("CARGO_NET_OFFLINE: \"true\""));
    assert!(prefetch.contains("kafka-wire = \"=0.1.0-rc.3\""));
    assert!(prefetch.contains("kafka-wire-core = \"=0.1.0-rc.3\""));
    assert!(prefetch.contains("bornera = \"=0.0.1-rc.3\""));
    assert!(prefetch.contains("bornera-core = \"=0.0.1-rc.3\""));
    assert!(prefetch.contains("bornera-rustls = \"=0.0.1-rc.3\""));
    assert!(prefetch.contains("cargo fetch --locked"));
    assert!(script.contains("cargo package"));
    assert!(script.contains("--no-verify"));
    assert!(script.contains("version=$(sed -n"));
    assert!(script.contains("$package-$version.crate"));
    assert_eq!(script.matches("cargo check").count(), 3);
    for package in [
        "kafka-driver-core",
        "kafka-driver-transport",
        "kafka-driver",
    ] {
        assert!(script.contains(package));
    }
    assert!(script.contains("normalized manifest retains a dependency path"));
    assert!(script.contains("cmp LICENSE"));
    assert!(script.contains("readme = \"README.md\""));
    assert!(script.contains("cmp README.md"));
    assert!(script.contains("cmp kafka-driver-logo.svg"));
    assert!(
        workflow.contains("name: qualified-crates-${{ github.sha }}-${{ github.run_attempt }}")
    );
    assert!(workflow.contains("path: kafka-driver/target/package/*.crate"));
    assert!(workflow.contains("if-no-files-found: error"));
    assert!(workflow.contains("retention-days: 90"));
}

pub(super) fn release_qualification_resolves_latest_compatible_from_a_clean_registry(root: &Path) {
    let workspace_root = || root.to_path_buf();
    let root = workspace_root();
    let workflow = read(&root.join(".github/workflows/qualification.yml"));
    let script = read(&root.join("scripts/qualify-latest-compatible"));

    assert!(workflow.contains("run: scripts/qualify-latest-compatible"));
    assert!(script.contains("version=$(sed -n"));
    assert!(script.contains("mktemp -d"));
    assert!(script.contains("export CARGO_HOME"));
    assert!(script.contains("unset CARGO_NET_OFFLINE"));
    assert_eq!(script.matches("cargo generate-lockfile").count(), 3);
    assert_eq!(script.matches("cargo check").count(), 3);
    assert!(script.contains("kafka-wire"));
    assert!(script.contains("kafka-wire-core"));
}

pub(super) fn release_qualification_is_scheduled_and_uses_the_registry_protocol(root: &Path) {
    let workspace_root = || root.to_path_buf();
    let root = workspace_root();
    let workflow = read(&root.join(".github/workflows/qualification.yml"));

    assert!(workflow.contains("schedule:"));
    assert!(workflow.contains("tags: [\"v*\"]"));
    assert!(!workflow.contains("kafka-protocol"));
    assert!(!workflow.contains("repository: kafkars/kafka-wire"));
    assert!(!workflow.contains("KAFKA_PROTOCOL_SSH_KEY"));
    assert!(!workflow.contains("KAFKA_PROTOCOL_TOKEN"));
    assert!(workflow.contains("run: npm run qualify:real-kafka"));
}
