//! Full frozen path selection and per-instance size mutations run only as trusted tests.

#[path = "native.rs"]
mod native;

use std::{collections::BTreeSet, fs, io::Write};

use serde_json::{Value, json};
use zrail_core::sha256_hex;

use super::{
    Budget, BudgetAllow, BudgetBaseline, legacy_driver, legacy_kafkars,
    model::{self, Snapshot},
};

pub(super) fn run() {
    let output = std::env::var_os("ZRAIL_RC9_SIZE_REPORT").expect("set fresh report output");
    let reports = [qualify("kafka-driver"), qualify("kafkars")];
    let project = model::project();
    let report = json!({
        "schema": 1, "status": "pass", "full_repository_qualified": false,
        "claim": "Physical Rust-file selection, line thresholds and exact size allowances only.",
        "implementation_commit": model::git(&project, &["rev-parse", "HEAD"]).trim(),
        "tracked_diff_sha256": sha256_hex(model::git(&project, &["diff", "HEAD", "--binary"]).as_bytes()),
        "reports": reports,
        "limitations": [
            "No Rust syntax, semantic resolution, compilation completeness or execution-evidence claim.",
            "No lock or source file is written; measured ratchet records are isolated evaluator inputs.",
            "Full repository contracts, approved locks and complete source analysis remain separate qualification.",
            "Standalone soft-limit warnings add observations without changing legacy size pass/fail."
        ]
    });
    let bytes = serde_json::to_vec_pretty(&report).expect("serialize deterministic size report");
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .expect("create fresh report")
        .write_all(&bytes)
        .expect("write report");
    println!("size-only parity pass: sha256:{}", sha256_hex(&bytes));
}

fn qualify(name: &str) -> Value {
    let snapshot = Snapshot::load(name);
    let roots = &snapshot.contract.repository.roots;
    let legacy_paths = if name == "kafka-driver" {
        legacy_driver::paths(&snapshot.root, roots)
    } else {
        let excluded = snapshot.config["paths"]["excluded_roots"]
            .as_array()
            .expect("exclusions")
            .iter()
            .map(|path| snapshot.root.join(path.as_str().expect("path")))
            .collect::<Vec<_>>();
        legacy_kafkars::paths(&snapshot.root, roots, &excluded)
    };
    let legacy_set = legacy_paths.iter().collect::<BTreeSet<_>>();
    let native_paths = snapshot
        .inventory
        .rust_files
        .iter()
        .map(|file| snapshot.root.join(&file.relative))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        legacy_paths.len(),
        legacy_set.len(),
        "no repeated legacy physical files"
    );
    assert!(
        legacy_set.iter().all(|path| native_paths.contains(*path))
            && legacy_set.len() == native_paths.len(),
        "complete physical selection differs"
    );
    let mut records = Vec::new();
    let mut matched_baselines = BTreeSet::new();
    let mut matched_allows = BTreeSet::new();
    let baselines: Vec<BudgetBaseline> = snapshot.config["budgets"]
        .get("baseline")
        .map(|value| value.clone().try_into().expect("frozen baselines"))
        .unwrap_or_default();
    let allows: Vec<BudgetAllow> = snapshot.config["budgets"]
        .get("allow")
        .map(|value| value.clone().try_into().expect("frozen allows"))
        .unwrap_or_default();
    assert_eq!(baselines.len(), snapshot.contract.ratchets.len());
    assert_eq!(
        allows.len(),
        snapshot
            .contract
            .source
            .rust
            .budgets
            .as_ref()
            .expect("budgets")
            .exceptions
            .len()
    );
    for file in &snapshot.inventory.rust_files {
        let budget = native::budget(file, &snapshot.contract.source.rust);
        let legacy = legacy_budget(name, &snapshot, &file.relative);
        assert_eq!(budget.thresholds.target, legacy.target, "{}", file.relative);
        assert_eq!(budget.thresholds.hard, legacy.hard, "{}", file.relative);
        if name == "kafkars" {
            assert_eq!(budget.thresholds.soft, Some(legacy.soft));
        }
        let baseline = baselines.iter().find(|entry| entry.path == file.relative);
        let allowance = allows.iter().find(|entry| entry.path == file.relative);
        let ratchet = snapshot
            .contract
            .ratchets
            .iter()
            .find(|entry| entry.target == file.relative);
        if let Some(entry) = baseline {
            matched_baselines.insert(entry.path.clone());
            let ratchet = ratchet.expect("translated baseline");
            assert_eq!(ratchet.baseline, Some(entry.lines));
            assert_eq!(ratchet.reason, entry.reason);
        }
        if let Some(entry) = allowance {
            matched_allows.insert(entry.path.clone());
            let exception = budget.exception.as_ref().expect("translated allowance");
            assert_eq!(exception.path, entry.path);
            assert_eq!(
                exception.hard,
                baseline
                    .expect("allow bounded by existing exact baseline")
                    .lines
            );
            assert_eq!(exception.reason, entry.reason);
            assert_eq!(exception.owner.as_deref(), Some(entry.owner.as_str()));
            assert_eq!(exception.issue.as_deref(), Some(entry.issue.as_str()));
        }
        let mut cases = vec![("valid", file.lines, None)];
        if baseline.is_some() {
            cases.extend([
                ("growth", file.lines + 1, Some("RUST-SIZE-009")),
                (
                    "shrink",
                    file.lines - 1,
                    Some(if file.lines - 1 <= legacy.target {
                        "RUST-SIZE-004"
                    } else {
                        "RUST-SIZE-009"
                    }),
                ),
                ("stale", legacy.target, Some("RUST-SIZE-004")),
            ]);
        } else {
            cases.extend([
                ("target-excess", legacy.target + 1, Some("RUST-SIZE-002")),
                ("hard-excess", legacy.hard + 1, Some("RUST-SIZE-001")),
            ]);
        }
        let mut outcomes = Vec::new();
        for (case, lines, expected) in cases {
            let old_errors = if name == "kafka-driver" {
                legacy_driver::violations(
                    &snapshot.root,
                    &snapshot.root.join(&file.relative),
                    driver_budgets(&snapshot),
                    lines,
                )
            } else {
                legacy_kafkars::violations(&file.relative, lines, legacy, baseline, allowance)
            };
            let errors = native::errors(
                file,
                lines,
                &budget,
                ratchet,
                &snapshot.contract.source.rust,
            );
            assert_eq!(
                old_errors.is_empty(),
                errors.is_empty(),
                "{} {case}: {old_errors:?} {errors:?}",
                file.relative
            );
            if let Some(expected) = expected {
                assert!(
                    errors.iter().any(|id| id == expected),
                    "{} {case}: {errors:?}",
                    file.relative
                );
            } else {
                assert!(errors.is_empty(), "{}: {errors:?}", file.relative);
            }
            outcomes.push(json!({"case": case, "lines": lines, "legacy_errors": old_errors, "native_error_ids": errors}));
        }
        if allowance.is_some() {
            let mut removed = budget.clone();
            removed.exception = None;
            let errors = native::errors(
                file,
                file.lines,
                &removed,
                ratchet,
                &snapshot.contract.source.rust,
            );
            assert!(errors.iter().any(|id| id == "RUST-SIZE-001"));
            let old =
                legacy_kafkars::violations(&file.relative, file.lines, legacy, baseline, None);
            assert!(!old.is_empty());
            outcomes.push(json!({"case": "missing-hard-allow", "legacy_errors": old, "native_error_ids": errors}));
        }
        records.push(
            json!({"path": file.relative, "source_sha256": sha256_hex(file.source.as_bytes()),
            "policy": budget, "baseline": ratchet, "outcomes": outcomes}),
        );
    }
    assert!(!records.is_empty());
    assert_eq!(matched_baselines.len(), baselines.len());
    assert_eq!(matched_allows.len(), allows.len());
    snapshot.verify();
    json!({"snapshot": snapshot.pin, "policy_sha256": snapshot.policy_sha256,
        "legacy_policy_sha256": snapshot.config_sha256, "physical_files": records.len(),
        "baseline_instances": matched_baselines.len(), "hard_allow_instances": matched_allows.len(),
        "files": records})
}

fn legacy_budget(name: &str, snapshot: &Snapshot, relative: &str) -> Budget {
    let path = snapshot.root.join(relative);
    if name == "kafka-driver" {
        let limit = legacy_driver::limit(&snapshot.root, &path, driver_budgets(snapshot));
        return Budget {
            target: limit,
            soft: limit,
            hard: limit,
        };
    }
    let manifest: toml::Value =
        toml::from_str(&fs::read_to_string(snapshot.root.join("Cargo.toml")).expect("manifest"))
            .expect("workspace manifest");
    let roots = manifest["workspace"]["members"]
        .as_array()
        .expect("literal frozen members")
        .iter()
        .map(|member| snapshot.root.join(member.as_str().expect("member path")))
        .collect::<Vec<_>>();
    let class = legacy_kafkars::classify_with_package_roots(&snapshot.root, &roots, &path);
    let key = match class {
        legacy_kafkars::FileClass::Facade => "facade",
        legacy_kafkars::FileClass::Test => "test",
        legacy_kafkars::FileClass::Implementation => "implementation",
        legacy_kafkars::FileClass::Auxiliary => "auxiliary",
    };
    snapshot.budgets()[key]
}

fn driver_budgets(snapshot: &Snapshot) -> legacy_driver::Budgets {
    let read = |name: &str| {
        usize::try_from(
            snapshot.config["budgets"][name]
                .as_integer()
                .expect("limit"),
        )
        .expect("positive bounded limit")
    };
    legacy_driver::Budgets {
        facade: read("facade"),
        production: read("production"),
        test: read("test"),
    }
}
