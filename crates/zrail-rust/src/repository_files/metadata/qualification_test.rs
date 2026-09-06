//! Trusted metadata runners bind frozen source, all fixture inputs, and the committed analyzer.

use std::{
    fs,
    io::Write as _,
    path::{Path, PathBuf},
    process::Command,
};

use zrail_core::sha256_hex;

use super::super::model::project;
use super::{fixtures, model};

#[test]
fn frozen_metadata_assertions_agree_on_every_authored_field_and_negative_input() {
    let root = std::env::temp_dir().join(format!(
        "zrail-metadata-parity-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let (policies, _) = model::policies(&project());
    let source = project().join("crates/zrail-testkit/tests/fixtures/rc9/metadata/valid");
    let inputs = model::inputs(&source, &policies);
    assert_eq!(inputs.len(), 11);
    let outcomes = fixtures::qualify(&root, &inputs, &policies);
    assert_eq!(
        outcomes.iter().filter(|row| row.native_accepted).count(),
        35
    );
    assert_eq!(outcomes.len(), 134);
}

#[test]
#[ignore = "requires prefetched snapshots and fresh ZRAIL_RC9_METADATA_REPORT"]
fn qualify_all_frozen_kafka_driver_metadata_assertions() {
    let project = project();
    let snapshots =
        PathBuf::from(std::env::var_os("ZRAIL_RC9_SNAPSHOTS").expect("prefetch snapshots"));
    let output =
        PathBuf::from(std::env::var_os("ZRAIL_RC9_METADATA_REPORT").expect("fresh output"));
    assert!(!output.exists(), "do not overwrite evidence");
    let evidence = fs::canonicalize(output.parent().expect("evidence parent"))
        .expect("existing evidence directory");
    assert!(!evidence.starts_with(fs::canonicalize(&project).expect("project root")));
    assert!(!evidence.starts_with(fs::canonicalize(&snapshots).expect("snapshots root")));
    let verified = Command::new("python3")
        .arg(project.join("scripts/rc9-snapshots"))
        .arg(&snapshots)
        .status()
        .expect("trusted offline snapshot verification");
    assert!(verified.success());
    clean(&project);
    let implementation_commit = git(&project, &["rev-parse", "HEAD"]);
    let pins: serde_json::Value = serde_json::from_slice(
        &fs::read(project.join("crates/zrail-testkit/tests/fixtures/rc9/snapshots.json"))
            .expect("source pins"),
    )
    .expect("parse pins");
    let snapshot = pins["consumers"]
        .as_array()
        .expect("consumer pins")
        .iter()
        .find(|pin| pin["name"] == "kafka-driver")
        .expect("Kafka pin")
        .clone();
    let source = snapshots.join("kafka-driver");
    verify(&source, &snapshot);
    let (policies, policy_sha256) = model::policies(&project);
    let inputs = model::inputs(&source, &policies);
    assert_eq!(inputs.len(), 11);
    let baseline = crate::repository_files::analyze(&source, &policies)
        .expect("complete frozen metadata selection");
    assert!(baseline.findings.is_empty(), "{:?}", baseline.findings);
    fixtures::legacy_result(&source)
        .expect("complete original metadata guard accepts the frozen source");
    let legacy = baseline
        .policies
        .iter()
        .flat_map(|policy| {
            policy.entries.iter().map(|entry| model::LegacyObservation {
                policy_id: policy.policy_id.clone(),
                path: entry.path.clone(),
                accepted: true,
            })
        })
        .collect();
    let fixture_root = evidence.join("metadata-parity-fixture");
    let fixtures = fixtures::qualify(&fixture_root, &inputs, &policies);
    assert_eq!(fixtures.len(), 134);
    assert_eq!(
        fixtures.iter().filter(|row| row.native_accepted).count(),
        35
    );
    let compiler = Command::new("rustc")
        .arg("--version")
        .current_dir(&project)
        .output()
        .expect("pinned compiler identity");
    assert!(compiler.status.success());
    let report = model::Report {
        schema: 1, implementation_commit, snapshot, inputs: model::hashes(&inputs),
        rustc_version: String::from_utf8(compiler.stdout).expect("compiler identity").trim().into(),
        test_binary_sha256: sha256_hex(&fs::read(std::env::current_exe().expect("test executable")).expect("test binary bytes")),
        cargo_lock_sha256: sha256_hex(&fs::read(project.join("Cargo.lock")).expect("compiler input lock")),
        fixture_origins_sha256: sha256_hex(&fs::read(project.join("crates/zrail-testkit/tests/fixtures/rc9/metadata-origins.json")).expect("frozen extraction origins")),
        policy_sha256, fixture_root: fixture_root.to_str().expect("UTF-8 fixture path").into(),
        predicates: policies.len(), full_repository_qualified: false,
        observations: baseline.policies, legacy, fixtures,
        limitations: vec![
            "Authored metadata fields, UTF-8 license equality, physical file presence, and deliberate raw markers only; no resolved Cargo or execution claim.".into(),
            "The original assertion body and helpers execute unchanged, with workspace_root supplied from the isolated input path.".into(),
            "Only the declared 11 metadata inputs are copied; other source and independently governed assertions require full repository qualification.".into(),
            "Malformed documents and exhausted bounds fail analysis explicitly instead of returning partial observations.".into(),
        ],
    };
    clean(&project);
    verify(&source, &report.snapshot);
    assert_eq!(
        git(&project, &["rev-parse", "HEAD"]),
        report.implementation_commit
    );
    assert_eq!(
        model::hashes(&model::inputs(&source, &policies)),
        report.inputs
    );
    let value = serde_json::to_value(&report).expect("canonical report");
    let mut bytes = serde_json::to_vec_pretty(&value).expect("serialize report");
    bytes.push(b'\n');
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&output)
        .expect("new report")
        .write_all(&bytes)
        .expect("write report");
    std::io::stdout()
        .write_all(
            format!(
                "metadata parity: {} policies, {} frozen inputs, {} fixtures; sha256 {}\n",
                report.predicates,
                report.inputs.len(),
                report.fixtures.len(),
                sha256_hex(&bytes)
            )
            .as_bytes(),
        )
        .expect("report summary");
}

fn verify(root: &Path, pin: &serde_json::Value) {
    for (expression, field) in [("HEAD", "commit"), ("HEAD^{tree}", "tree")] {
        assert_eq!(
            git(root, &["rev-parse", expression]),
            pin[field].as_str().expect("source identity")
        );
    }
    clean(root);
}

fn clean(root: &Path) {
    assert!(
        git(root, &["status", "--porcelain=v1", "--untracked-files=all"]).is_empty(),
        "qualify committed inputs"
    );
}

fn git(root: &Path, arguments: &[&str]) -> String {
    let result = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(arguments)
        .output()
        .expect("trusted Git observation");
    assert!(result.status.success());
    String::from_utf8(result.stdout)
        .expect("Git output")
        .trim()
        .into()
}
