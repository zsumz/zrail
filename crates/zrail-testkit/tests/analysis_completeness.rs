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
    assert!(checked.candidate_lock.is_some());
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
