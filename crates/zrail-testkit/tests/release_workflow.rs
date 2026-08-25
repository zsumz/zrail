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
    assert!(workflow.contains("(-[0-9A-Za-z-]+(\\.[0-9A-Za-z-]+)*)?"));
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
    assert!(workflow.contains("zrail init --preset rust --baseline /work"));
    assert!(!workflow.contains("zrail init --preset rust /work"));
    assert!(publish.contains("sha256sum --check SHA256SUMS"));
    assert!(publish.contains("actions/attest@"));
    assert!(publish.contains("release-notes.py"));
    assert!(publish.contains("scripts/release-state.py prepare"));
    assert!(publish.contains("scripts/release-state.py publish"));
    assert_eq!(workflow.matches("contents: write").count(), 1);
}

#[test]
fn crate_publication_is_ordered_exact_and_safe_to_resume() {
    let workflow = release_workflow();
    let workflow_publish = section(&workflow, "  publish:", "__end_of_workflow__");
    let publish = publish_script();
    let setup = workflow_publish
        .find("Set up the same pinned publish Rust")
        .expect("publish toolchain setup");
    let preflight = workflow_publish
        .find("scripts/publish-crates --preflight")
        .expect("publish-mode archive preflight");
    let draft = workflow_publish
        .find("Prepare or resume the exact draft release")
        .expect("draft preparation");
    let registry_publish = workflow_publish
        .find("scripts/publish-crates --publish")
        .expect("registry publication");
    let byte_proof = publish
        .find("# A separate pre-publication step")
        .expect("publish-mode byte proof");
    let vendor = publish
        .find("cargo vendor --locked --versioned-dirs")
        .expect("versioned directory source");
    let dry_run = publish
        .find("publish --package \"$package\" --locked --no-verify --dry-run")
        .expect("publish dry-run");
    let stage = publish
        .find("python3 scripts/stage-crate-source.py")
        .expect("predecessor archive staging");
    let registry_probe = publish.find("published=()").expect("registry probe");
    let cargo_upload = publish
        .rfind("cargo publish --package \"$package\" --locked --no-verify --registry crates-io")
        .expect("Cargo registry upload");

    assert!(workflow.contains("rust-lang/crates-io-auth-action@"));
    assert!(setup < preflight && preflight < draft && draft < registry_publish);
    assert_eq!(
        workflow_publish
            .matches("uses: ./.github/actions/setup-rust")
            .count(),
        1
    );
    assert!(publish.contains("packages=(zrail-core zrail-rust zrail)"));
    assert!(publish.contains("cargo \"${publish_args[@]}\""));
    assert!(publish.contains("source.crates-io.replace-with = 'zrail-publish-source'"));
    assert!(publish.contains("--registry crates-io \"${preflight_source_config[@]}\""));
    assert!(publish.contains("--locked-dependency zrail-core"));
    assert!(publish.contains("--locked-dependency zrail-rust"));
    assert!(!publish.contains("patch.crates-io"));
    assert!(!publish.contains("package_roots="));
    assert!(vendor < dry_run && dry_run < stage && stage < registry_probe);
    assert!(publish.contains("pinned publish toolchain"));
    assert!(publish.contains("compare_preflight_archives"));
    assert!(publish.contains("target/package/tmp-crate/$package-$ZRAIL_VERSION.crate"));
    assert!(publish.contains("require_matching_generated_archive \"$package\""));
    assert!(byte_proof < registry_probe && registry_probe < cargo_upload);
    assert_eq!(
        publish[cargo_upload..]
            .lines()
            .next()
            .expect("Cargo upload line"),
        "cargo publish --package \"$package\" --locked --no-verify --registry crates-io"
    );
    assert!(publish.contains("for attempt in {1..30}"));
    assert!(publish.contains("if [[ \"${published[$index]}\" == yes ]]"));
    assert!(publish.contains("cmp \"dist/$package-$ZRAIL_VERSION.crate\" \"$generated\""));
    assert!(publish.contains("download_exact_registry_archive \"$package\""));
}

#[test]
fn packaged_archives_keep_and_check_their_registry_locks() {
    let package = package_check_script();

    assert!(package.contains("cargo vendor --locked --offline --versioned-dirs"));
    assert!(package.contains("source.crates-io.replace-with = 'zrail-package-source'"));
    assert!(package.contains("python3 scripts/stage-crate-source.py"));
    assert!(package.contains("--locked-dependency zrail-core"));
    assert!(package.contains("--locked-dependency zrail-rust"));
    assert!(package.contains("cargo check --manifest-path \"$package_dir/Cargo.toml\" --locked"));
    assert!(!package.contains("patch.crates-io"));
    assert!(!package.contains("cp Cargo.lock"));
}

#[test]
fn partial_publication_reruns_reuse_only_the_exact_draft() {
    let workflow = release_workflow();
    let publish_job = section(&workflow, "  publish:", "__end_of_workflow__");
    let draft = section(
        publish_job,
        "- name: Prepare or resume the exact draft release",
        "- name: Authenticate with crates.io trusted publishing",
    );
    let publisher = publish_script();
    let state = release_state_script();

    assert!(draft.contains("scripts/release-state.py prepare"));
    assert!(draft.contains("--release-id-file \"$RUNNER_TEMP/release-id\""));
    assert!(state.contains("release(tagName:$tag){databaseId}"));
    assert!(state.contains("repository_url(f\"releases/{release_id}\")"));
    assert!(!state.contains("releases/tags/"));
    assert!(state.contains("remote tag does not peel to GITHUB_SHA"));
    assert!(state.contains("\"body\": self.body"));
    assert!(state.contains("actual != expected_bytes"));
    assert!(state.contains("\"prerelease\": self.prerelease"));
    assert!(state.contains("{\"draft\": False}"));
    assert!(!state.contains("target_commitish"));
    for reviewed_argument in [
        "--assets-dir dist",
        "--assets-file \"$RUNNER_TEMP/release-assets.txt\"",
        "--notes-file \"$RUNNER_TEMP/release-notes.md\"",
        "--release-id-file \"$RUNNER_TEMP/release-id\"",
        "--version \"$ZRAIL_VERSION\"",
        "--title \"zrail $ZRAIL_VERSION\"",
    ] {
        assert_eq!(publish_job.matches(reviewed_argument).count(), 2);
    }
    assert!(publisher.contains("200)"));
    assert!(publisher.contains("published+=(yes)"));
    assert!(publisher.contains("if [[ \"${published[$index]}\" == yes ]]"));
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

fn package_check_script() -> String {
    fs::read_to_string(repository_root().join("scripts/package-check"))
        .expect("read crate package checker")
}

fn release_state_script() -> String {
    fs::read_to_string(repository_root().join("scripts/release-state.py"))
        .expect("read GitHub release-state helper")
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
