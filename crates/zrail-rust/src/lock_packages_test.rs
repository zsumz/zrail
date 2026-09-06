//! Trusted frozen whole-lock parity never produces a source-analysis certificate or executes Cargo.

#[path = "lock_packages/parity/fixtures.rs"]
mod fixtures;
#[path = "lock_packages/parity/legacy.rs"]
mod legacy;
#[path = "lock_packages/parity/model.rs"]
mod model;
#[path = "lock_packages/parity/mutations.rs"]
mod mutations;

#[test]
fn frozen_lock_provenance_preserves_counts_and_complete_immutable_identities() {
    let root = std::env::temp_dir().join(format!(
        "zrail-lock-parity-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let (rules, _) = model::policies();
    let inputs = model::inputs(
        &model::project().join("crates/zrail-testkit/tests/fixtures/rc9/lock-packages/valid"),
    );
    let outcomes = fixtures::qualify(&root, &inputs, &rules);
    assert_eq!(outcomes.len(), 50);
    assert_eq!(
        outcomes.iter().filter(|row| row.native_accepted).count(),
        20
    );
    for rule in rules {
        let rows = outcomes
            .iter()
            .filter(|row| row.policy_id == format!("dependency:lock-package:{}", rule.name))
            .collect::<Vec<_>>();
        assert_eq!(rows.len(), 10);
        assert_eq!(rows.iter().filter(|row| !row.native_accepted).count(), 6);
    }
}

use std::{
    fs,
    io::Write as _,
    path::{Path, PathBuf},
    process::Command,
};

use serde_json::json;
use zrail_core::sha256_hex;

#[test]
#[ignore = "requires prefetched snapshots and fresh ZRAIL_RC9_LOCK_PACKAGES_REPORT"]
fn qualify_all_frozen_kafka_driver_lock_inventories() {
    let project = model::project();
    let snapshots =
        PathBuf::from(std::env::var_os("ZRAIL_RC9_SNAPSHOTS").expect("prefetched snapshots"));
    let output =
        PathBuf::from(std::env::var_os("ZRAIL_RC9_LOCK_PACKAGES_REPORT").expect("fresh output"));
    assert!(!output.exists(), "do not overwrite evidence");
    let evidence =
        fs::canonicalize(output.parent().expect("evidence parent")).expect("evidence directory");
    assert!(!evidence.starts_with(fs::canonicalize(&project).expect("project root")));
    assert!(!evidence.starts_with(fs::canonicalize(&snapshots).expect("snapshot root")));
    assert!(
        Command::new("python3")
            .arg(project.join("scripts/rc9-snapshots"))
            .arg(&snapshots)
            .status()
            .expect("trusted offline snapshot verification")
            .success()
    );
    clean(&project);
    let commit = git(&project, &["rev-parse", "HEAD"]);
    let tree = git(&project, &["rev-parse", "HEAD^{tree}"]);
    let pins: serde_json::Value = serde_json::from_slice(
        &fs::read(project.join("crates/zrail-testkit/tests/fixtures/rc9/snapshots.json"))
            .expect("source pins"),
    )
    .expect("parse pins");
    let snapshot = pins["consumers"]
        .as_array()
        .expect("pins")
        .iter()
        .find(|pin| pin["name"] == "kafka-driver")
        .expect("driver snapshot")
        .clone();
    let source = snapshots.join("kafka-driver");
    verify(&source, &snapshot);
    let workspace = model::workspace(&source);
    let (rules, policy_sha256) = model::policies();
    let inputs = model::inputs(&source);
    assert_eq!(inputs.len(), 12);
    let observations =
        model::native(&source, &workspace, &rules).expect("complete frozen lock graph");
    assert!(observations.iter().all(|policy| policy.satisfied));
    fixtures::legacy_result(&source).expect("original frozen guard accepts the snapshot");
    let fixture_root = evidence.join("lock-packages-parity-fixture");
    let outcomes = fixtures::qualify(&fixture_root, &inputs, &rules);
    assert_eq!(outcomes.len(), 50);
    assert_eq!(
        outcomes.iter().filter(|row| row.native_accepted).count(),
        20
    );
    let compiler = Command::new("rustc")
        .arg("--version")
        .current_dir(&project)
        .output()
        .expect("pinned compiler");
    assert!(compiler.status.success());
    let legacy = observations
        .iter()
        .map(|policy| json!({"policy_id":policy.policy_id,"path":"Cargo.lock","accepted":true}))
        .collect::<Vec<_>>();
    let report = json!({
        "schema":1, "implementation_commit":commit, "implementation_tree":tree,
        "snapshot":snapshot, "policy_sha256":policy_sha256, "inputs":model::hashes(&inputs),
        "cargo_inventory_roots":["."],
        "workspace_packages":workspace.iter().map(|package| json!({"name":package.name,"directory":package.directory})).collect::<Vec<_>>(),
        "projection_contract_sha256":sha256_hex(&fs::read(project.join("crates/zrail-testkit/tests/fixtures/good/zrail.toml")).expect("projection fixture contract")),
        "rustc_version":String::from_utf8(compiler.stdout).expect("compiler identity").trim(),
        "test_binary_sha256":sha256_hex(&fs::read(std::env::current_exe().expect("test executable")).expect("test binary bytes")),
        "cargo_lock_sha256":sha256_hex(&fs::read(project.join("Cargo.lock")).expect("compiler dependency lock")),
        "fixture_origins_sha256":sha256_hex(&fs::read(project.join("crates/zrail-testkit/tests/fixtures/rc9/lock-packages-origins.json")).expect("frozen origins")),
        "fixture_root":fixture_root.to_str().expect("UTF-8 fixture path"),
        "predicates":rules.len(), "full_repository_qualified":false,
        "observations":observations, "legacy":legacy, "fixtures":outcomes,
        "limitations":[
            "Whole-lock cardinality and complete version/source/checksum sets only. Native Cargo projection derives all five workspace identities from the frozen manifests and target presence.",
            "The original complete protocol lock test body and its read/parse/assert_locked helpers execute unchanged; workspace_root alone is supplied from the isolated fixture path.",
            "Fixtures copy and bind all five manifests, the registry, Cargo.lock, and five original target roots. Source bodies and the source module graph are not analyzed; no source certificate, candidate lock, or full repository qualification is produced.",
            "Deletion, version/source substitutions, and duplicate-name fixtures explicitly adjust incoming dependency references to keep the graph resolvable. All failures reach DEP-LOCK-001 for the intended package rule.",
            "The manifest registry stays pinned as translation authority. Its exact-version-prefix guards and the legacy lock-array/name type preconditions remain separate reviewed assertions.",
        ],
    });
    clean(&project);
    verify(&source, &report["snapshot"]);
    assert_eq!(
        git(&project, &["rev-parse", "HEAD"]),
        report["implementation_commit"].as_str().expect("code SHA")
    );
    assert_eq!(
        git(&project, &["rev-parse", "HEAD^{tree}"]),
        report["implementation_tree"].as_str().expect("code tree")
    );
    assert_eq!(
        model::hashes(&model::inputs(&source)),
        model::hashes(&inputs)
    );
    let mut bytes = serde_json::to_vec_pretty(&report).expect("canonical report");
    bytes.push(b'\n');
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&output)
        .expect("new report")
        .write_all(&bytes)
        .expect("write evidence");
    std::io::stdout()
        .write_all(
            format!(
                "lock provenance parity: 5 policies, 12 frozen inputs, 50 fixtures; sha256 {}\n",
                sha256_hex(&bytes)
            )
            .as_bytes(),
        )
        .expect("qualification summary");
}

fn verify(root: &Path, pin: &serde_json::Value) {
    for (expression, field) in [("HEAD", "commit"), ("HEAD^{tree}", "tree")] {
        assert_eq!(
            git(root, &["rev-parse", expression]),
            pin[field].as_str().expect("snapshot identity")
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
