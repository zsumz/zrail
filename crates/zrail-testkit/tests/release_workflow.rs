//! Release workflow structure is explicit, complete, and protected.

use std::{fs, path::PathBuf};

#[test]
fn release_requires_protected_tagged_source_before_repository_execution() {
    let workflow = release_workflow();
    let ancestry = workflow
        .find("git merge-base --is-ancestor")
        .expect("tag ancestry check");
    let local_action = workflow
        .find("uses: ./.github/actions/setup-rust")
        .expect("local Rust action");

    assert!(workflow.contains("tags: [\"v*\"]"));
    assert!(!workflow.contains("pull_request:"));
    assert!(!workflow.contains("workflow_dispatch:"));
    assert_eq!(workflow.matches("environment: release").count(), 2);
    assert!(ancestry < local_action);
    assert!(workflow.contains("test \"$version\" = \"$manifest_version\""));
    assert!(workflow.contains("Run complete qualification gate offline"));
    assert!(workflow.contains("run: scripts/check"));
    assert!(!workflow.contains("--accept-grants"));
    assert!(!workflow.contains("--allow-grants"));
}

#[test]
fn release_covers_the_exact_native_target_and_archive_set() {
    let workflow = release_workflow();
    let targets = [
        "x86_64-unknown-linux-gnu",
        "aarch64-unknown-linux-gnu",
        "x86_64-unknown-linux-musl",
        "aarch64-unknown-linux-musl",
        "x86_64-apple-darwin",
        "aarch64-apple-darwin",
        "x86_64-pc-windows-msvc",
    ];
    for target in targets {
        assert!(workflow.contains(target), "missing release target {target}");
    }

    assert!(workflow.contains("runner: macos-15-intel"));
    assert!(workflow.contains("runner: macos-15"));
    assert!(workflow.contains("runner: windows-2025"));
    assert!(workflow.contains("runner: ubuntu-24.04-arm"));
    assert!(workflow.contains("binary=\"$binary.exe\""));
    assert!(workflow.contains(".tar.gz"));
    assert!(workflow.contains(".zip"));
    assert!(workflow.contains("test \"$(\"$binary\" --version)\""));
    assert!(workflow.contains("package-release.py"));
}

#[test]
fn publishing_waits_for_all_builds_and_clean_linux_runtime_checks() {
    let workflow = release_workflow();
    let publish = section(&workflow, "  publish:", "__end_of_workflow__");

    assert!(workflow.contains("needs: [qualify, build]"));
    assert!(publish.contains("needs: [qualify, build, package, linux-runtime]"));
    assert!(workflow.contains("ubuntu:24.04"));
    assert!(workflow.contains("alpine:3.22"));
    assert!(workflow.contains("Run without a Rust toolchain in a clean container"));
    assert!(workflow.contains("docker run --rm"));
    assert!(publish.contains("sha256sum --check SHA256SUMS"));
    assert!(publish.contains("actions/attest@"));
    assert!(publish.contains("release-notes.py"));
    assert!(publish.contains("gh release create \"$GITHUB_REF_NAME\""));
    assert!(publish.contains("--verify-tag --draft"));
    assert!(publish.contains("gh release edit \"$GITHUB_REF_NAME\" --draft=false"));
    assert_eq!(workflow.matches("contents: write").count(), 1);
}

#[test]
fn crate_publication_is_ordered_exact_and_safe_to_resume() {
    let workflow = release_workflow();
    let publish = publish_script();

    assert!(workflow.contains("rust-lang/crates-io-auth-action@"));
    assert!(workflow.contains("run: scripts/publish-crates"));
    assert!(publish.contains("packages=(zrail-core zrail-rust zrail)"));
    assert!(publish.contains("for attempt in {1..30}"));
    assert!(publish.contains("if [[ \"${published[$index]}\" == yes ]]"));
    assert!(publish.contains("cmp \"dist/$package-$ZRAIL_VERSION.crate\" \"$generated\""));
    assert!(publish.contains("download_exact_registry_archive \"$package\""));
}

#[test]
fn every_external_release_action_uses_an_immutable_full_sha() {
    for line in release_workflow().lines() {
        let Some(reference) = line.trim().strip_prefix("uses: ") else {
            continue;
        };
        let reference = reference
            .split_whitespace()
            .next()
            .expect("action reference");
        if reference.starts_with("./") {
            continue;
        }
        let (_, revision) = reference.split_once('@').expect("action revision");
        assert_eq!(revision.len(), 40, "action is not full-SHA pinned: {line}");
        assert!(
            revision.bytes().all(|byte| byte.is_ascii_hexdigit()),
            "action is not hex-SHA pinned: {line}"
        );
    }
}

#[test]
fn readme_uses_verified_prebuilt_binaries_for_ci() {
    let readme = fs::read_to_string(repository_root().join("README.md")).expect("read README");
    let binary = readme
        .find("Download verified zrail binary")
        .expect("binary CI example");
    let fallback = readme.find("### Cargo fallback").expect("Cargo fallback");

    assert!(binary < fallback);
    assert!(readme.contains("releases/download/v${ZRAIL_VERSION}"));
    assert!(readme.contains("sha256sum --check"));
    assert!(readme.contains("gh attestation verify"));
    assert!(!section(&readme, "CI should use", "### Cargo fallback").contains("cargo install"));
}

fn release_workflow() -> String {
    fs::read_to_string(repository_root().join(".github/workflows/release.yml"))
        .expect("read release workflow")
}

fn publish_script() -> String {
    fs::read_to_string(repository_root().join("scripts/publish-crates"))
        .expect("read crate publisher")
}

fn section<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start = source.find(start).expect("section start");
    let tail = &source[start..];
    let end = tail.find(end).unwrap_or(tail.len());
    &tail[..end]
}

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("testkit lives below repository root")
        .to_path_buf()
}
