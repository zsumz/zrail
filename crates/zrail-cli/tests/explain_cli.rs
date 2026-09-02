//! End-to-end path explanation behavior.

use std::{path::PathBuf, process::Command};

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../zrail-testkit/tests/fixtures/good")
}

#[test]
fn nonexistent_concrete_path_exits_nonzero_without_classification() {
    let output = Command::new(env!("CARGO_BIN_EXE_zrail"))
        .args([
            "explain",
            "--root",
            fixture_root().to_str().expect("UTF-8 fixture path"),
            "--path",
            "crates/fixture/src/workre.rs",
        ])
        .output()
        .expect("run zrail explain");

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let error = String::from_utf8(output.stderr).expect("UTF-8 error output");
    assert!(error.contains("path does not exist"));
    assert!(error.contains("interpreted as repository-relative"));
    assert!(error.contains("crates/fixture/src/worker.rs"));
}

#[test]
fn hypothetical_path_remains_available_explicitly() {
    let output = Command::new(env!("CARGO_BIN_EXE_zrail"))
        .args([
            "explain",
            "--root",
            fixture_root().to_str().expect("UTF-8 fixture path"),
            "--hypothetical-path",
            "crates/fixture/src/future.rs",
        ])
        .output()
        .expect("run zrail explain");

    assert!(output.status.success());
    let report = String::from_utf8(output.stdout).expect("UTF-8 report output");
    assert!(report.contains("crates/fixture/src/future.rs"));
    assert!(output.stderr.is_empty());
}
