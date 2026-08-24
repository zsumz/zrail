//! Incomplete source analysis is observable but can never construct lock authority.

use std::{fs, path::PathBuf};

use zrail_core::ReportStatus;
use zrail_rust::{AnalysisIssueKind, build_lock, check_repository};

#[test]
fn parse_failure_returns_invalid_report_without_candidate_lock() {
    let root = fixture("incomplete-parse", "pub fn broken( {\n");

    let checked = check_repository(&root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("report incomplete analysis");

    assert_eq!(checked.report.status, ReportStatus::Invalid);
    assert!(checked.candidate_lock.is_none());
    assert!(!checked.analysis.is_complete());
    assert_eq!(
        checked.analysis.issues()[0].kind,
        AnalysisIssueKind::SourceInput
    );
    let error = build_lock(&root, "zrail.toml".as_ref()).expect_err("refuse partial lock");
    assert!(error.to_string().contains("incomplete analysis"));
    reset(&root);
}

#[test]
fn complete_analysis_returns_an_in_memory_candidate() {
    let root = fixture("complete-source", "//! Complete source.\n");

    let checked = check_repository(&root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check complete analysis");

    assert!(checked.analysis.is_complete());
    let certificate = checked
        .candidate_lock
        .as_ref()
        .and_then(|lock| lock.analysis.as_ref())
        .expect("complete candidate certificate");
    assert_eq!(certificate.packages, 1);
    assert_eq!(certificate.targets, 1);
    assert_eq!(certificate.physical_rust_files, 1);
    assert_eq!(certificate.unresolved_bindings, 0);
    assert_eq!(certificate.contract_sources.len(), 1);
    assert!(checked.report.analysis.complete);
    assert_eq!(checked.report.analysis.rust_files, 1);
    reset(&root);
}

#[test]
fn completeness_certificate_binds_exact_cargo_lock_bytes() {
    let root = fixture("cargo-lock-certificate", "//! Complete source.\n");
    let lock = "version = 3\n\n[[package]]\nname = \"fixture\"\nversion = \"0.0.0\"\n";
    fs::write(root.join("Cargo.lock"), lock).expect("write Cargo.lock");
    let before = build_lock(&root, "zrail.toml".as_ref()).expect("build first lock");
    fs::write(root.join("Cargo.lock"), format!("{lock}\n")).expect("change Cargo.lock bytes");
    let after = build_lock(&root, "zrail.toml".as_ref()).expect("build second lock");

    let before = before.analysis.expect("first certificate");
    let after = after.analysis.expect("second certificate");
    assert!(before.cargo_lock_sha256.is_some());
    assert_ne!(before.cargo_lock_sha256, after.cargo_lock_sha256);
    assert_ne!(before.inventory_sha256, after.inventory_sha256);
    reset(&root);
}

fn fixture(name: &str, source: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("zrail-analysis-{name}-{}", std::process::id()));
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create fixture");
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname='fixture'\nversion='0.0.0'\nedition='2024'\n",
    )
    .expect("write manifest");
    fs::write(root.join("src/lib.rs"), source).expect("write source");
    fs::write(root.join("zrail.toml"), contract()).expect("write contract");
    root
}

fn contract() -> &'static str {
    r#"schema = 1
adapters = ["rust"]

[repository]
roots = ["."]
exclude = []
workspace_members = "exact"
nested_git = "deny"
submodules = "deny"
symlinks = "inside"

[dependencies]
mode = "observed"
unassigned_packages = "allow"
cycles = "deny"

[source.rust]
module_docs = "allow"
facades = "allow"
tests = "allow"

[source.rust.hygiene]
unsafe = "allow"
lint_suppressions = "allow"
deny_methods = []
deny_macros = []
"#
}

fn reset(root: &std::path::Path) {
    if root.exists() {
        fs::remove_dir_all(root).expect("remove fixture");
    }
}
