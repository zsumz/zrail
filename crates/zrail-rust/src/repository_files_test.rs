//! Frozen raw detectors qualify native file predicates, independently of Rust or execution evidence.

#[path = "repository_files/parity/baseline.rs"]
mod baseline;
#[path = "repository_files/dependency_fields/qualification.rs"]
mod dependency_fields;
#[path = "repository_files/parity/fixtures.rs"]
mod fixtures;
#[path = "repository_files/key_sets/qualification.rs"]
mod key_sets;
#[path = "repository_files/parity/legacy.rs"]
mod legacy;
#[path = "repository_files/metadata/qualification.rs"]
mod metadata;
#[path = "repository_files/parity/model.rs"]
mod model;
#[path = "repository_files/provenance/qualification.rs"]
mod provenance;
#[path = "repository_files/qualification/qualification.rs"]
mod qualification;
#[path = "repository_files/raw_dependency/qualification.rs"]
mod raw_dependency;

use std::{
    fs,
    io::Write as _,
    path::{Path, PathBuf},
    process::Command,
};

#[test]
fn frozen_file_predicates_agree_on_positive_and_adversarial_physical_fixtures() {
    let root = std::env::temp_dir().join(format!(
        "zrail-file-parity-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let (policies, _) = model::policies();
    let rows = fixtures::qualify(&root, &policies);
    for policy in policies {
        let rows = rows
            .iter()
            .filter(|row| row.policy_id == format!("repository:file:{}", policy.name))
            .collect::<Vec<_>>();
        assert!(rows.iter().any(|row| row.legacy_accepted));
        assert!(rows.iter().any(|row| !row.legacy_accepted));
    }
}

#[test]
#[ignore = "requires explicitly prefetched frozen snapshots and fresh ZRAIL_RC9_FILE_REPORT"]
fn qualify_all_frozen_kafka_driver_file_predicates() {
    let project = model::project();
    let snapshots =
        PathBuf::from(std::env::var_os("ZRAIL_RC9_SNAPSHOTS").expect("prefetch snapshots"));
    let output =
        PathBuf::from(std::env::var_os("ZRAIL_RC9_FILE_REPORT").expect("fresh report path"));
    assert!(!output.exists(), "do not replace reviewed evidence");
    let evidence_root = fs::canonicalize(output.parent().expect("report directory"))
        .expect("existing external evidence directory");
    assert!(!evidence_root.starts_with(fs::canonicalize(&project).expect("project root")));
    assert!(!evidence_root.starts_with(fs::canonicalize(&snapshots).expect("snapshot root")));
    let status = Command::new("python3")
        .arg(project.join("scripts/rc9-snapshots"))
        .arg(&snapshots)
        .status()
        .expect("trusted snapshot and extraction verification");
    assert!(status.success());
    assert!(
        git(
            &project,
            &["status", "--porcelain=v1", "--untracked-files=all"]
        )
        .is_empty(),
        "qualify committed code"
    );
    let commit = git(&project, &["rev-parse", "HEAD"]);
    let snapshot = model::Snapshot::load(snapshots.join("kafka-driver"));
    for (expression, key) in [("HEAD", "commit"), ("HEAD^{tree}", "tree")] {
        assert_eq!(
            git(&snapshot.root, &["rev-parse", expression]),
            snapshot.pin[key].as_str().expect("source pin")
        );
    }
    let (policies, policy_sha256) = model::policies();
    let analysis =
        super::analyze(&snapshot.root, &policies).expect("complete frozen file selection");
    assert!(analysis.findings.is_empty(), "{:?}", analysis.findings);
    let legacy = baseline::compare(&snapshot, &analysis.policies);
    let fixture_root = evidence_root.join("file-parity-fixture");
    let fixtures = fixtures::qualify(&fixture_root, &policies);
    let report = model::Report {
        schema: 1, implementation_commit: commit, snapshot: snapshot.pin,
        rustc_version: {
            let result = Command::new("rustc").arg("--version").current_dir(&project).output().expect("pinned compiler identity");
            assert!(result.status.success());
            String::from_utf8(result.stdout).expect("compiler identity").trim().into()
        },
        test_binary_sha256: zrail_core::sha256_hex(&fs::read(std::env::current_exe().expect("test executable")).expect("test binary bytes")),
        cargo_lock_sha256: zrail_core::sha256_hex(&fs::read(project.join("Cargo.lock")).expect("locked compiler inputs")),
        fixture_origins_sha256: zrail_core::sha256_hex(&fs::read(project.join("crates/zrail-testkit/tests/fixtures/rc9/file-origins.json")).expect("extraction origins")),
        legacy_policy_sha256: snapshot.config_sha256, policy_sha256,
        fixture_root: fixture_root.to_str().expect("UTF-8 fixture context").into(),
        predicates: policies.len(), full_repository_qualified: false,
        observations: analysis.policies, legacy, fixtures,
        limitations: vec![
            "Only physical path and raw UTF-8 predicates are qualified; no Rust resolution, compilation, lock, or execution claim.".into(),
            "Positive text predicates require a nonempty selection; the mission explicitly requires absent positive subjects to fail.".into(),
            "Unread subtrees and escaping/unresolved file links fail closed; no directory-link or cache exclusions were added to the frozen policy.".into(),
        ],
    };
    assert!(
        git(
            &snapshot.root,
            &["status", "--porcelain=v1", "--untracked-files=all"]
        )
        .is_empty()
    );
    assert!(
        git(
            &project,
            &["status", "--porcelain=v1", "--untracked-files=all"]
        )
        .is_empty()
    );
    assert_eq!(
        git(&project, &["rev-parse", "HEAD"]),
        report.implementation_commit
    );
    for (expression, key) in [("HEAD", "commit"), ("HEAD^{tree}", "tree")] {
        assert_eq!(
            git(&snapshot.root, &["rev-parse", expression]),
            report.snapshot[key].as_str().expect("source pin")
        );
    }
    let value = serde_json::to_value(&report).expect("canonical report value");
    let mut bytes = serde_json::to_vec_pretty(&value).expect("deterministic report JSON");
    bytes.push(b'\n');
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&output)
        .expect("new report")
        .write_all(&bytes)
        .expect("write qualification evidence");
    std::io::stdout().write_all(format!("file parity: {} policies, {} frozen occurrences, {} adversarial fixtures; sha256 {}\n",
        report.predicates, report.legacy.len(), report.fixtures.len(), zrail_core::sha256_hex(&bytes)).as_bytes()).expect("qualification summary");
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
        .expect("UTF-8 Git output")
        .trim()
        .into()
}
