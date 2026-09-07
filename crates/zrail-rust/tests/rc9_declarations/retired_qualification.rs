//! Slice evidence binds clean committed producer, untouched frozen inputs and deterministic outcomes.

use super::{
    cases,
    model::{self, BACKEND, CONSTRUCTION, Fixture, MODULES, POLICY},
};
use serde_json::json;
use std::{
    collections::BTreeMap,
    fs,
    io::Write as _,
    path::{Path, PathBuf},
    process::Command,
};
use zrail_core::sha256_hex;

pub(super) fn run() {
    let project = model::project();
    let snapshots =
        PathBuf::from(std::env::var_os("ZRAIL_RC9_SNAPSHOTS").expect("prefetched snapshots"))
            .canonicalize()
            .expect("snapshots");
    let output = PathBuf::from(std::env::var_os("ZRAIL_RC9_RETIRED_REPORT").expect("fresh report"));
    let directory = output
        .parent()
        .expect("parent")
        .canonicalize()
        .expect("existing evidence directory");
    assert!(!output.exists());
    assert!(!directory.starts_with(project.canonicalize().expect("project")));
    assert!(!directory.starts_with(&snapshots));
    let verify = || {
        assert!(
            Command::new("python3")
                .arg(project.join("scripts/rc9-snapshots"))
                .arg(&snapshots)
                .status()
                .expect("snapshot verification")
                .success()
        );
        assert!(
            git(
                &project,
                &["status", "--porcelain=v1", "--untracked-files=all"]
            )
            .is_empty()
        );
    };
    verify();
    let commit = git(&project, &["rev-parse", "HEAD"]);
    let pins: serde_json::Value = serde_json::from_slice(
        &fs::read(project.join("crates/zrail-testkit/tests/fixtures/rc9/snapshots.json"))
            .expect("pins"),
    )
    .expect("parse pins");
    let pin = pins["consumers"]
        .as_array()
        .expect("consumers")
        .iter()
        .find(|pin| pin["name"] == "kafka-driver")
        .expect("snapshot");
    let snapshot = snapshots.join("kafka-driver");
    let inputs: BTreeMap<String, String> = [MODULES, BACKEND, CONSTRUCTION]
        .into_iter()
        .map(|path| {
            (
                path.into(),
                fs::read_to_string(snapshot.join(path)).expect("frozen UTF-8 input"),
            )
        })
        .collect();
    let fixture = Fixture::at(directory.join("retired-parity-fixture"), inputs);
    let policies = model::policies();
    let baseline = model::observe(&fixture, &policies, "valid");
    assert!(baseline.native_accepted);
    let original_files = crate::repository_files::analyze(&snapshot, &policies.repository.files)
        .expect("complete snapshot tree discovery");
    assert!(original_files.findings.is_empty());
    assert_eq!(
        serde_json::to_value(&original_files.policies).expect("snapshot observations"),
        serde_json::to_value(&baseline.files).expect("fixture observations")
    );
    let rows = cases::run(&fixture, &policies);
    let rustc = Command::new("rustc")
        .arg("-Vv")
        .output()
        .expect("toolchain");
    assert!(rustc.status.success());
    let report = json!({
        "schema": 1, "implementation_commit": commit,
        "implementation_tree": git(&project, &["rev-parse", "HEAD^{tree}"]),
        "snapshot": pin, "full_repository_qualified": false,
        "policy_sha256": sha256_hex(&fs::read(project.join(POLICY)).expect("policy")),
        "fixture_origins_sha256": sha256_hex(&fs::read(project.join("crates/zrail-testkit/tests/fixtures/rc9/declarations-origins.json")).expect("origins")),
        "test_binary_sha256": sha256_hex(&fs::read(std::env::current_exe().expect("binary")).expect("binary bytes")),
        "cargo_lock_sha256": sha256_hex(&fs::read(project.join("Cargo.lock")).expect("lock")),
        "rustc_version": String::from_utf8(rustc.stdout).expect("compiler identity").trim(),
        "input_hashes": fixture.hashes(), "fixture_root": fixture.root,
        "baseline": baseline, "fixtures": rows,
        "limitations": [
            "Only the release-graph source predicates and their required inputs are qualified; no full Cargo, lock or downstream execution claim.",
            "The old tree walker ignores read errors and follows directory links without bounds. Native unread and link boundaries fail closed; this report does not equate those failure paths.",
            "All frozen source checks remain installed; no downstream cutover or authority acceptance is performed."
        ]
    });
    drop(fixture);
    verify();
    assert_eq!(git(&project, &["rev-parse", "HEAD"]), commit);
    let mut bytes = serde_json::to_vec_pretty(&report).expect("report");
    bytes.push(b'\n');
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .expect("new report")
        .write_all(&bytes)
        .expect("write evidence");
    writeln!(
        std::io::stderr(),
        "retired parity: {} cases; payload {}",
        report["fixtures"].as_array().expect("rows").len(),
        sha256_hex(&bytes)
    )
    .expect("summary");
}

fn git(root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .expect("git");
    assert!(output.status.success());
    String::from_utf8(output.stdout)
        .expect("UTF-8 git output")
        .trim()
        .into()
}
