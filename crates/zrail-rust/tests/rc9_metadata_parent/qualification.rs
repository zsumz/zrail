//! Frozen three-path proof with complete original execution and no general manifest claim.

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
    origins_sha256: String,
    original_metadata_body_executed: bool,
    original_parent_instances: usize,
    full_repository_qualified: bool,
    inputs: BTreeMap<String, String>,
    path_proof: Vec<model::PathProof>,
    observations: Vec<crate::GovernedRepositoryFile>,
    limitations: [&'static str; 3],
}

pub(super) fn run() {
    let project = model::project();
    let snapshots = PathBuf::from(std::env::var_os("ZRAIL_RC9_SNAPSHOTS").expect("snapshots"))
        .canonicalize()
        .expect("frozen directory");
    let output =
        PathBuf::from(std::env::var_os("ZRAIL_RC9_METADATA_PARENT_REPORT").expect("fresh report"));
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
        fs::read(root.join("tests/guardrails/release_metadata.rs")).expect("original source");
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
    let report = Report {
        schema: 1,
        snapshot,
        source_sha256: sha256_hex(&source),
        policy_sha256: sha256_hex(&fs::read(project.join(model::POLICY)).expect("existing policy")),
        origins_sha256: sha256_hex(&fs::read(project.join(model::ORIGINS)).expect("origins")),
        original_metadata_body_executed: true,
        original_parent_instances: 3,
        full_repository_qualified: false,
        inputs: model::inputs(&root),
        path_proof: model::path_proof(&root),
        observations: model::original_and_native(&root),
        limitations: [
            "One lexical parent precondition over three fixed authored manifest paths; not arbitrary manifest discovery or full metadata qualification.",
            "Twelve path mappings use four absolute anchor shapes; synthetic anchors are lexical only, not filesystem or permission observations.",
            "Existing UTF-8 license policies are reused unchanged; missing, unreadable or unequal licenses remain governed separately, not parent-precondition failures.",
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
