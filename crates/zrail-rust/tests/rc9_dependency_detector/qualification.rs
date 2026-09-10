//! The trusted runner binds exact execution and producer identity around this native report.

use super::{model, original};
use serde::Serialize;
use std::{fs, io::Write as _, path::PathBuf};
use zrail_core::sha256_hex;

#[derive(Serialize)]
struct Report {
    schema: u32,
    snapshot: serde_json::Value,
    source_sha256: String,
    original_detector_sha256: String,
    original_helper_sha256: String,
    cases_sha256: String,
    policy_sha256: String,
    original_assertions_executed: usize,
    full_repository_qualified: bool,
    fixtures: Vec<model::Row>,
    limitations: [&'static str; 3],
}

pub(super) fn run() {
    let project = model::project();
    let snapshots = PathBuf::from(std::env::var_os("ZRAIL_RC9_SNAPSHOTS").expect("snapshots"))
        .canonicalize()
        .expect("frozen directory");
    let output = PathBuf::from(
        std::env::var_os("ZRAIL_RC9_DEPENDENCY_DETECTOR_REPORT").expect("fresh report"),
    );
    let directory = output
        .parent()
        .expect("parent")
        .canonicalize()
        .expect("external directory");
    assert!(
        !output.exists() && !directory.starts_with(&project) && !directory.starts_with(&snapshots)
    );
    let source = fs::read(snapshots.join("kafka-driver/tests/guardrails/dependency.rs"))
        .expect("frozen source");
    assert_eq!(sha256_hex(&source), model::SOURCE_SHA);
    let pins: serde_json::Value = serde_json::from_slice(
        &fs::read(project.join("crates/zrail-testkit/tests/fixtures/rc9/snapshots.json"))
            .expect("pins"),
    )
    .expect("pins JSON");
    let snapshot = pins["consumers"]
        .as_array()
        .expect("consumers")
        .iter()
        .find(|pin| pin["name"] == "kafka-driver")
        .expect("snapshot")
        .clone();
    model::source_binding();
    original::check();
    let report = Report {
        schema: 1,
        snapshot,
        source_sha256: sha256_hex(&source),
        original_detector_sha256: model::DETECTOR_SHA.into(),
        original_helper_sha256: model::HELPER_SHA.into(),
        cases_sha256: sha256_hex(&fs::read(project.join(model::CASES)).expect("cases")),
        policy_sha256: sha256_hex(&fs::read(project.join(model::POLICY)).expect("policy")),
        original_assertions_executed: 2,
        full_repository_qualified: false,
        fixtures: model::qualify(),
        limitations: [
            "Two inline detector assertions only; fixture policies are not an installed consumer contract.",
            "Raw lines deliberately inspect unrelated sections, malformed TOML and multiline strings; no package graph or TOML parsing claim.",
            "Original name sets deduplicate; native complete-line counts preserve multiplicity and bound offset samples independently of membership.",
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
