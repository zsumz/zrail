//! Frozen original execution and exact native array failures do not claim complete cutover.

use super::model;
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
    empty_counts: Vec<model::EmptyResult>,
    observations: Vec<crate::GovernedLockPackage>,
    limitations: [&'static str; 4],
}

pub(super) fn run() {
    let project = model::project();
    let snapshots = PathBuf::from(std::env::var_os("ZRAIL_RC9_SNAPSHOTS").expect("snapshots"))
        .canonicalize()
        .expect("frozen directory");
    let output =
        PathBuf::from(std::env::var_os("ZRAIL_RC9_LOCK_ARRAY_REPORT").expect("fresh report"));
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
        empty_counts: model::empty_counts(),
        observations: model::frozen(&root),
        limitations: [
            "One array-type precondition only; the separate all-entry name precondition, exact-version authority and full repository qualification remain open.",
            "Missing or non-array package fields fail the original and existing native parser; an empty array passes this type stage but all five required package counts fail separately.",
            "Synthetic graph cases deliberately use no active workspace packages; the frozen run derives all five actual workspace packages from twelve bound inputs.",
            "Earlier lock-version and later package-field validation can reject array-typed inputs; these are existing native constraints, not array-precondition parity or new grants.",
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
