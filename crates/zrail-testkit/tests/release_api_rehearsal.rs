//! The GitHub release helper survives and validates real API-state transitions.

#![cfg(unix)]

use std::{path::PathBuf, process::Command};

#[test]
fn mocked_github_release_state_machine_is_fail_closed_and_resumable() {
    let root = repository_root();
    let rehearsal = root.join("crates/zrail-testkit/tests/release_api_rehearsal/rehearsal.py");
    let status = Command::new("python3")
        .arg("-B")
        .arg(rehearsal)
        .arg(root.join("scripts/release-state.py"))
        .status()
        .expect("run mocked GitHub release rehearsal");
    assert!(status.success());
}

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("testkit lives below repository root")
        .to_path_buf()
}
