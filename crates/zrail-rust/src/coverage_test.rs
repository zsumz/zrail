//! Governed-surface reporting stays complete, exact, and deterministic.

use std::{fs, path::PathBuf};

use zrail_core::AnalysisQuality;

use super::governed_surface_report;
use fixture::{AMBIGUOUS_LOCK, CHECKSUM, CONTRACT, LIBRARY, LOCK, MANIFEST, MIRROR, OWNER};

#[path = "coverage/fixture.rs"]
mod fixture;

#[test]
fn report_covers_operations_dependencies_exclusions_and_test_mirrors() {
    let root = repository("complete");

    let report = governed_surface_report(&root, "zrail.toml".as_ref()).expect("build coverage");
    let repeated = governed_surface_report(&root, "zrail.toml".as_ref()).expect("repeat coverage");

    assert_eq!(report, repeated);
    assert_eq!(report.schema, 3);
    assert_eq!(report.contract_schema, 2);
    assert_eq!(report.contract_sha256.len(), 64);
    assert!(report.analysis.complete);
    assert_eq!(report.analysis.metrics.physical_rust_files, 3);
    assert_eq!(report.analysis.exclusions, ["scratch/**", "target/**"]);
    assert!(
        report
            .enabled_rails
            .windows(2)
            .all(|pair| pair[0] < pair[1])
    );
    for rail in [
        "adapter:rust",
        "analysis:limit:derived-source-instances:input-derived",
        "contract-source:zrail.toml",
        "dependency:blocked-path",
        "owner:type-construction:record-construction",
        "repository:exclude:scratch/**",
        "rust:hygiene:unsafe",
        "test-mirror:tests/mirror.rs::mirrors_build",
    ] {
        assert!(
            report.enabled_rails.iter().any(|enabled| enabled == rail),
            "missing enabled rail {rail:?}"
        );
    }
    assert_eq!(report.owners.len(), 5);
    assert_eq!(report.unresolved_occurrences, 1);
    assert_eq!(report.ambiguous_occurrences, 0);
    let owner = report
        .owners
        .iter()
        .find(|owner| owner.name == "record-construction")
        .expect("construction owner");
    assert_eq!(
        owner.policy_id,
        "owner:type-construction:record-construction"
    );
    let occurrence = &owner.occurrences[0];
    assert_eq!(occurrence.path, "src/owner.rs");
    assert_eq!(occurrence.quality, AnalysisQuality::Exact);
    assert_eq!(occurrence.guard, "ordinary");
    assert!(occurrence.span.is_some());
    assert!(occurrence.allowed);
    assert!(
        occurrence
            .compilation_domains
            .iter()
            .any(|domain| domain.mode == "library")
    );
    let field_owner = report
        .owners
        .iter()
        .find(|owner| owner.name == "unknown-token-authority")
        .expect("field authority owner");
    assert_eq!(
        field_owner.policy_id,
        "owner:field-authority:unknown-token-authority"
    );
    assert_eq!(field_owner.occurrences[0].operation, "field-write");
    let call_owner = report
        .owners
        .iter()
        .find(|owner| owner.name == "metadata-call")
        .expect("call owner");
    assert_eq!(
        call_owner
            .occurrences
            .iter()
            .map(|occurrence| occurrence.operation.as_str())
            .collect::<Vec<_>>(),
        ["direct-call", "reference"]
    );
    let capability_owner = report
        .owners
        .iter()
        .find(|owner| owner.name == "environment-use")
        .expect("capability owner");
    assert_eq!(capability_owner.occurrences[0].operation, "capability-use");
    let directory_owner = report
        .owners
        .iter()
        .find(|owner| owner.name == "artifact-directories")
        .expect("directory owner");
    assert_eq!(
        directory_owner
            .occurrences
            .iter()
            .map(|occurrence| (occurrence.path.as_str(), occurrence.allowed))
            .collect::<Vec<_>>(),
        [("artifacts/owned", true), ("artifacts/trespass", false)]
    );
    assert!(
        directory_owner
            .occurrences
            .iter()
            .all(
                |occurrence| occurrence.compilation_domains.is_empty() && occurrence.span.is_none()
            )
    );
    assert_eq!(report.dependencies.len(), 1);
    let path = &report.dependencies[0].paths[0];
    assert_eq!(path.kind, "normal");
    assert_eq!(path.denied, "blocked");
    assert_eq!(
        path.nodes
            .iter()
            .map(|node| node.name.as_str())
            .collect::<Vec<_>>(),
        ["audit-app", "bridge", "blocked"]
    );
    assert_eq!(path.nodes[2].checksum.as_deref(), Some(CHECKSUM));
    assert_eq!(
        path.nodes[2].source,
        "registry+https://github.com/rust-lang/crates.io-index"
    );
    assert_eq!(report.test_mirrors[0].test_name, "mirrors_build");
    assert_eq!(report.test_mirrors[0].package, "audit-app");
    assert_eq!(
        report.test_mirrors[0].inputs,
        ["Cargo.lock", "Cargo.toml", "src/owner.rs"]
    );
    assert_eq!(report.test_mirrors[0].target, "x86_64-unknown-linux-gnu");
    assert_eq!(report.json().expect("json"), repeated.json().expect("json"));
    assert!(report.json().expect("json").contains("\"enabled_rails\""));
    assert!(report.human().contains("dependency:blocked-path"));
    assert!(report.human().contains("Enabled rails:"));
    reset(&root);
}

#[test]
fn report_rejects_incomplete_source_analysis() {
    let root = repository("incomplete");
    write(&root, "src/owner.rs", "pub fn broken(\n");

    let error = governed_surface_report(&root, "zrail.toml".as_ref())
        .expect_err("incomplete analysis must not produce coverage");

    assert!(
        error
            .to_string()
            .contains("coverage requires complete analysis")
    );
    assert!(error.to_string().contains("RUST-PARSE"));
    reset(&root);
}

#[test]
fn report_rejects_ambiguous_manifest_to_lock_mapping() {
    let root = repository("ambiguous");
    write(
        &root,
        "Cargo.toml",
        &MANIFEST.replace("bridge = \"1\"", "bridge = \"*\""),
    );
    write(&root, "Cargo.lock", AMBIGUOUS_LOCK);

    let error = governed_surface_report(&root, "zrail.toml".as_ref())
        .expect_err("ambiguous lock mapping must not produce coverage");

    assert!(error.to_string().contains("maps ambiguously"));
    assert!(error.to_string().contains("bridge 1.2.3"));
    assert!(error.to_string().contains("bridge 2.0.0"));
    reset(&root);
}

fn repository(label: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "zrail-coverage-{label}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create source");
    fs::create_dir_all(root.join("tests")).expect("create tests");
    fs::create_dir_all(root.join("evidence")).expect("create evidence");
    fs::create_dir_all(root.join("artifacts/owned")).expect("create owned directory");
    fs::create_dir_all(root.join("artifacts/trespass")).expect("create trespass directory");
    write(&root, "Cargo.toml", MANIFEST);
    write(&root, "Cargo.lock", LOCK);
    write(&root, "zrail.toml", CONTRACT);
    write(&root, "src/lib.rs", LIBRARY);
    write(&root, "src/owner.rs", OWNER);
    write(&root, "tests/mirror.rs", MIRROR);
    write(&root, "evidence/mirror.json", "{}\n");
    root
}

fn write(root: &std::path::Path, path: &str, contents: &str) {
    fs::write(root.join(path), contents).expect("write fixture");
}

fn reset(root: &std::path::Path) {
    if root.exists() {
        fs::remove_dir_all(root).expect("reset fixture");
    }
}
