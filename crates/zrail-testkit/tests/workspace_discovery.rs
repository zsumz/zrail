//! Cargo analysis is bounded to the active workspace before package resolution.

use std::path::{Path, PathBuf};

use zrail_core::ReportStatus;
use zrail_rust::check_repository;

#[test]
fn excluded_sibling_workspace_does_not_enter_cargo_or_source_analysis() {
    let root = fixture_root();

    let checked = check_repository(&root, Path::new("zrail.toml"), Path::new("zrail.lock"))
        .expect("check active workspace");

    assert_eq!(
        checked.report.status,
        ReportStatus::Pass,
        "{}",
        checked.report.human()
    );
    assert_eq!(checked.packages, 1);
    assert_eq!(checked.rust_files, 1);
}

fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/workspace_discovery")
}
