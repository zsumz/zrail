//! Frozen original execution and exact native name failures do not claim complete cutover.

use super::{model, mutations};
use serde::Serialize;
use std::{collections::BTreeMap, fs, io::Write as _, path::PathBuf};
use zrail_core::sha256_hex;

#[derive(Serialize)]
struct Report {
    schema: u32,
    snapshot: serde_json::Value,
    source_sha256: String,
    policy_sha256: String,
    cases_sha256: String,
    origins_sha256: String,
    original_origins_sha256: String,
    original_helper_invocations: usize,
    full_repository_qualified: bool,
    inputs: BTreeMap<String, String>,
    workspace_packages: BTreeMap<String, String>,
    fixtures: Vec<model::Row>,
    mutations_sha256: String,
    mutations: Vec<mutations::Row>,
    observations: Vec<crate::GovernedLockPackage>,
    limitations: [&'static str; 4],
}

pub(super) fn run() {
    let project = model::project();
    let snapshots = PathBuf::from(std::env::var_os("ZRAIL_RC9_SNAPSHOTS").expect("snapshots"))
        .canonicalize()
        .expect("frozen directory");
    let output =
        PathBuf::from(std::env::var_os("ZRAIL_RC9_LOCK_NAMES_REPORT").expect("fresh report"));
    let directory = output
        .parent()
        .expect("parent")
        .canonicalize()
        .expect("external directory");
    assert!(
        !output.exists() && !directory.starts_with(&project) && !directory.starts_with(&snapshots)
    );
    let root = snapshots.join("kafka-driver");
    let source =
        fs::read(root.join("tests/guardrails/protocol_provenance.rs")).expect("original source");
    assert_eq!(sha256_hex(&source), model::SOURCE_SHA);
    let pins: serde_json::Value = serde_json::from_slice(
        &fs::read(project.join("crates/zrail-testkit/tests/fixtures/rc9/snapshots.json"))
            .expect("pins"),
    )
    .expect("JSON");
    let snapshot = pins["consumers"]
        .as_array()
        .expect("consumers")
        .iter()
        .find(|pin| pin["name"] == "kafka-driver")
        .expect("pin")
        .clone();
    model::source_binding();
    let workspace_packages = model::lock_model::workspace(&root)
        .into_iter()
        .map(|package| (package.name, package.directory))
        .collect();
    let report = Report {
        schema: 1,
        snapshot,
        source_sha256: sha256_hex(&source),
        policy_sha256: sha256_hex(&fs::read(project.join(model::POLICY)).expect("existing policy")),
        cases_sha256: sha256_hex(&fs::read(project.join(model::CASES)).expect("cases")),
        origins_sha256: sha256_hex(&fs::read(project.join(model::ORIGINS)).expect("origins")),
        original_origins_sha256: sha256_hex(
            &fs::read(
                project.join("crates/zrail-testkit/tests/fixtures/rc9/lock-packages-origins.json"),
            )
            .expect("complete original origins"),
        ),
        original_helper_invocations: 5,
        full_repository_qualified: false,
        inputs: model::lock_model::hashes(&model::lock_model::inputs(&root)),
        workspace_packages,
        fixtures: model::matrix(),
        mutations_sha256: sha256_hex(&fs::read(project.join(model::MUTATIONS)).expect("mutations")),
        mutations: mutations::run(&root),
        observations: model::frozen(&root),
        limitations: [
            "One all-entry name-index precondition only; version-prefix authority and full repository qualification remain open.",
            "Missing names or non-table entries fail original whole-array indexing; present non-string and empty names are ignored originally but rejected by existing native parsing.",
            "Synthetic selection cases have no active workspace packages; frozen and mutation runs derive all five real packages from twelve bound inputs and preserve whole-lock scope.",
            "Native case-sensitive matching does not normalize whitespace, case or dashes; later version and duplicate-node errors are not name-precondition parity or new grants.",
        ],
    };
    let mut bytes = serde_json::to_vec_pretty(&serde_json::to_value(report).expect("typed report"))
        .expect("JSON");
    bytes.push(b'\n');
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .expect("fresh output")
        .write_all(&bytes)
        .expect("report");
}
