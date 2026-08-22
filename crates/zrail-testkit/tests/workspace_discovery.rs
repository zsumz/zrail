//! Cargo analysis is bounded to the active workspace before package resolution.

use std::path::{Path, PathBuf};

use zrail_rust::check_repository;

#[test]
fn unlisted_package_is_reported_without_entering_ignored_workspaces() {
    let root = fixture_root();

    let checked = check_repository(&root, Path::new("zrail.toml"), Path::new("zrail.lock"))
        .expect("check active workspace");

    assert!(finding(&checked.report, "DEP-001", "rogue/Cargo.toml"));
    assert!(finding(
        &checked.report,
        "RUST-GRAPH-004",
        "rogue/src/lib.rs"
    ));
    assert!(!checked.report.findings.iter().any(|finding| {
        finding
            .path
            .as_deref()
            .is_some_and(|path| path.starts_with("reference/") || path.starts_with("sandbox/"))
    }));
    assert_eq!(checked.packages, 2);
    assert_eq!(checked.rust_files, 3);
}

fn finding(report: &zrail_core::Report, id: &str, path: &str) -> bool {
    report
        .findings
        .iter()
        .any(|finding| finding.id == id && finding.path.as_deref() == Some(path))
}

fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/workspace_discovery")
}
