//! Repeatable frozen route qualification remains outside the non-executing analyzer.

use std::{fs, io::Write as _, path::PathBuf, process::Command};

use serde_json::json;
use zrail_core::sha256_hex;

use super::{
    compiler::{Compiler, SOURCE, SOURCE_SHA},
    fixtures,
    model::{self, Fixture},
};

pub(super) fn run() {
    let project = model::project();
    let snapshots =
        PathBuf::from(std::env::var_os("ZRAIL_RC9_SNAPSHOTS").expect("prefetched snapshots"));
    let output =
        PathBuf::from(std::env::var_os("ZRAIL_RC9_ROUTE_REPORT").expect("fresh report path"));
    assert!(!output.exists());
    let directory =
        fs::canonicalize(output.parent().expect("report parent")).expect("evidence directory");
    assert!(!directory.starts_with(fs::canonicalize(&project).expect("project")));
    assert!(!directory.starts_with(fs::canonicalize(&snapshots).expect("snapshots")));
    let verify = || {
        assert!(
            Command::new("python3")
                .arg(project.join("scripts/rc9-snapshots"))
                .arg(&snapshots)
                .status()
                .expect("offline snapshot verification")
                .success()
        );
        assert!(
            super::super::git(
                &project,
                &["status", "--porcelain=v1", "--untracked-files=all"]
            )
            .is_empty()
        );
    };
    verify();
    let commit = super::super::git(&project, &["rev-parse", "HEAD"]);
    let tree = super::super::git(&project, &["rev-parse", "HEAD^{tree}"]);
    let snapshot = super::super::model::Snapshot::load(snapshots.join("kafka-driver"));
    assert_eq!(sha256_hex(snapshot.sources[SOURCE].as_bytes()), SOURCE_SHA);
    let policies = model::policies();
    let fixture = Fixture::at(directory.join("route-parity-fixture"), &policies);
    for (path, source) in &fixture.inputs {
        assert_eq!(
            source, &snapshot.sources[path],
            "exact frozen input: {path}"
        );
    }
    let baseline = fixtures::inputs(&fixture);
    let compiler = Compiler::new(&fixture);
    let rows = fixtures::qualify(&fixture, &policies, &compiler);
    assert_eq!(fixtures::inputs(&fixture), baseline);
    let rustc = Command::new("rustc")
        .arg("--version")
        .current_dir(&project)
        .output()
        .expect("compiler identity");
    assert!(rustc.status.success());
    let report = json!({
        "schema": 1, "implementation_commit": commit, "implementation_tree": tree,
        "snapshot": snapshot.pin, "full_repository_qualified": false,
        "rustc_version": String::from_utf8(rustc.stdout).expect("compiler").trim(),
        "test_binary_sha256": sha256_hex(&fs::read(std::env::current_exe().expect("test binary")).expect("binary bytes")),
        "cargo_lock_sha256": sha256_hex(&fs::read(project.join("Cargo.lock")).expect("Cargo lock")),
        "fixture_origins_sha256": sha256_hex(&fs::read(project.join("crates/zrail-testkit/tests/fixtures/rc9/route-sources-origins.json")).expect("origins")),
        "policy_sha256": sha256_hex(&fs::read(project.join("docs/rc9/policies/kafka-driver.route-sources.fragment.toml")).expect("policy bytes")),
        "source_test_sha256": SOURCE_SHA, "harness_sha256": compiler.harness_sha256,
        "input_hashes": baseline, "fixture_root": fixture.root, "fixtures": rows,
        "limitations": [
            "Only the source-only route test and its eleven compile inputs are qualified; the mixed file's runtime scenario is retained and unqualified here.",
            "Raw UTF-8 and physical-file observations do not claim semantic field/type identity, Cargo analysis, or full repository qualification.",
            "The complete unchanged source-only test compiles and executes on the baseline. Each missing or invalid UTF-8 input is compiled separately and must fail at its original include_str invocation."
        ]
    });
    drop(fixture);
    verify();
    assert_eq!(super::super::git(&project, &["rev-parse", "HEAD"]), commit);
    let mut bytes = serde_json::to_vec_pretty(&report).expect("deterministic report");
    bytes.push(b'\n');
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .expect("new evidence")
        .write_all(&bytes)
        .expect("write evidence");
    writeln!(
        std::io::stderr(),
        "route parity: 189 cases, 43 accepted, 146 rejected, 23 original compilations; payload {}",
        sha256_hex(&bytes)
    )
    .expect("summary");
}
