//! Direct-call ownership conformance, including conservative bypass cases.

use std::path::{Path, PathBuf};

use zrail_core::{AnalysisQuality, Finding};
use zrail_rust::check_repository;

#[test]
fn direct_call_outside_its_owner_is_rejected() {
    let report = check();
    let finding = find(&report.findings, "OWN-003", "trespasser.rs");
    assert_eq!(finding.analysis, AnalysisQuality::Exact);
}

#[test]
fn glob_import_cannot_escape_a_call_owner() {
    let report = check();
    let finding = find(&report.findings, "OWN-003", "glob_trespasser.rs");
    assert_eq!(finding.analysis, AnalysisQuality::Conservative);
}

#[test]
fn indirect_invocation_inside_an_owner_is_unverifiable() {
    let report = check();
    assert_finding(&report.findings, "OWN-005", "indirect_owner.rs");
}

#[test]
fn conservative_call_inside_an_owner_is_unverifiable() {
    let report = check();
    let finding = find(&report.findings, "OWN-005", "uncertain_owner.rs");
    assert_eq!(finding.analysis, AnalysisQuality::Conservative);
}

#[test]
fn unused_allowed_call_owner_is_stale() {
    let report = check();
    assert_finding(&report.findings, "OWN-004", "worker.rs");
}

fn assert_finding(findings: &[Finding], id: &str, file: &str) {
    let _ = find(findings, id, file);
}

fn find<'a>(findings: &'a [Finding], id: &str, file: &str) -> &'a Finding {
    findings
        .iter()
        .find(|finding| {
            finding.id == id
                && finding.rule == "filesystem-metadata-call"
                && finding
                    .path
                    .as_deref()
                    .is_some_and(|path| path.rsplit('/').next() == Some(file))
        })
        .unwrap_or_else(|| panic!("missing {id} for {file}"))
}

fn check() -> zrail_core::Report {
    let root = fixture_root();
    check_repository(&root, Path::new("zrail.toml"), Path::new("zrail.lock"))
        .unwrap_or_else(|error| panic!("check {}: {error}", root.display()))
        .report
}

fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join("forbidden_capability")
}
