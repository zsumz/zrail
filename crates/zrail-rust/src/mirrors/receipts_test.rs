//! Bulk receipt rendering binds every outcome to one canonical plan and group.

use std::collections::BTreeMap;

use zrail_core::{ExecutionReceiptStatus, TestExecutionIdentity, TestMirrorContract, sha256_hex};

use super::{MirrorExecutionResult, MirrorReceiptBundle, MirrorResultSet, MirrorTestResult};
use crate::{AnalysisMetrics, mirrors::model::PlannedTestMirror};

#[test]
fn renders_every_exact_result_as_a_schema_two_receipt() {
    let plan = plan();
    let results = MirrorResultSet {
        schema: 1,
        plan_sha256: plan.plan_sha256.clone(),
        producer: "trusted-runner 1.2.3".into(),
        groups: vec![group(
            &plan,
            [
                ExecutionReceiptStatus::Passed,
                ExecutionReceiptStatus::Failed,
            ],
        )],
    };

    let bundle = MirrorReceiptBundle::render(&plan, results).expect("render receipts");

    assert_eq!(bundle.schema, 1);
    assert_eq!(bundle.receipts.len(), 2);
    assert_eq!(bundle.receipts[0].path, "receipts/a.json");
    let first = zrail_core::parse_execution_receipt(&bundle.receipts[0].source)
        .expect("parse first receipt");
    assert_eq!(first.schema, 2);
    let second = zrail_core::parse_execution_receipt(&bundle.receipts[1].source)
        .expect("parse second receipt");
    assert_eq!(second.tests[0].status, ExecutionReceiptStatus::Failed);
    for artifact in &bundle.receipts {
        assert_eq!(artifact.sha256, sha256_hex(artifact.source.as_bytes()));
    }
}

#[test]
fn rejects_stale_missing_unknown_and_wrong_group_results() {
    let plan = plan();
    let mut stale = valid_results(&plan);
    stale.plan_sha256 = "0".repeat(64);
    assert!(MirrorReceiptBundle::render(&plan, stale).is_err());

    let mut missing = valid_results(&plan);
    missing.groups[0].tests.pop();
    assert!(MirrorReceiptBundle::render(&plan, missing).is_err());

    let mut unknown = valid_results(&plan);
    unknown.groups[0].tests[0].policy_id = "unknown".into();
    assert!(MirrorReceiptBundle::render(&plan, unknown).is_err());

    let mut wrong_group = valid_results(&plan);
    wrong_group.groups[0].execution_group = "group-b".into();
    assert!(MirrorReceiptBundle::render(&plan, wrong_group).is_err());
}

#[test]
fn parser_requires_canonical_groups_tests_and_versioned_producer() {
    let plan = plan();
    let mut results = valid_results(&plan);
    results.groups[0].tests.reverse();
    let reversed = serde_json::to_string(&results).expect("serialize results");
    assert!(MirrorResultSet::parse(&reversed).is_err());

    results.groups[0].tests.reverse();
    results.producer = "unversioned".into();
    let producer = serde_json::to_string(&results).expect("serialize results");
    assert!(MirrorResultSet::parse(&producer).is_err());
}

#[test]
fn renders_1244_mirrors_across_multiple_execution_groups_deterministically() {
    let mut entries = Vec::new();
    let mut grouped = BTreeMap::<String, Vec<MirrorTestResult>>::new();
    for index in 0..1_244 {
        let group = index % 7;
        let mirror = TestMirrorContract {
            production: format!("src/production_{index:04}.rs"),
            test: format!("tests/group_{group}.rs"),
            name: format!("mirror_{index:04}"),
            receipt: format!("evidence/mirror_{index:04}.json"),
            inputs: vec!["Cargo.lock".into(), "Cargo.toml".into()],
            execution: TestExecutionIdentity {
                command: format!("cargo test -p app --test group_{group}"),
                package: "app".into(),
                default_features: group % 2 == 0,
                features: vec![format!("group-{group}")],
                target: "x86_64-unknown-linux-gnu".into(),
                toolchain: "rustc 1.96.0".into(),
            },
            reason: "Synthetic bulk mirror".into(),
        };
        let entry =
            PlannedTestMirror::new(&mirror, format!("{index:064x}")).expect("planned bulk mirror");
        grouped
            .entry(entry.execution_group.clone())
            .or_default()
            .push(MirrorTestResult {
                policy_id: entry.policy_id.clone(),
                status: ExecutionReceiptStatus::Passed,
            });
        entries.push(entry);
    }
    entries.sort_by(|left, right| left.policy_id.cmp(&right.policy_id));
    for tests in grouped.values_mut() {
        tests.sort_by(|left, right| left.policy_id.cmp(&right.policy_id));
    }
    let plan = crate::MirrorPlan::new("b".repeat(64), AnalysisMetrics::default(), entries)
        .expect("bulk plan");
    let results = MirrorResultSet {
        schema: 1,
        plan_sha256: plan.plan_sha256.clone(),
        producer: "trusted-runner 1.2.3".into(),
        groups: grouped
            .into_iter()
            .map(|(execution_group, tests)| MirrorExecutionResult {
                execution_group,
                tests,
            })
            .collect(),
    };

    let first = MirrorReceiptBundle::render(&plan, results.clone()).expect("first bulk render");
    let second = MirrorReceiptBundle::render(&plan, results).expect("second bulk render");
    assert_eq!(first, second);
    assert_eq!(first.receipts.len(), 1_244);
    assert!(
        first
            .receipts
            .windows(2)
            .all(|pair| pair[0].path < pair[1].path)
    );
}

fn valid_results(plan: &crate::MirrorPlan) -> MirrorResultSet {
    MirrorResultSet {
        schema: 1,
        plan_sha256: plan.plan_sha256.clone(),
        producer: "trusted-runner 1.2.3".into(),
        groups: vec![group(
            plan,
            [
                ExecutionReceiptStatus::Passed,
                ExecutionReceiptStatus::Passed,
            ],
        )],
    }
}

fn group(plan: &crate::MirrorPlan, statuses: [ExecutionReceiptStatus; 2]) -> MirrorExecutionResult {
    MirrorExecutionResult {
        execution_group: plan.mirrors[0].execution_group.clone(),
        tests: plan
            .mirrors
            .iter()
            .zip(statuses)
            .map(|(mirror, status)| MirrorTestResult {
                policy_id: mirror.policy_id.clone(),
                status,
            })
            .collect(),
    }
}

fn plan() -> crate::MirrorPlan {
    let execution = TestExecutionIdentity {
        command: "cargo test --package app".into(),
        package: "app".into(),
        default_features: true,
        features: vec!["strict".into()],
        target: "x86_64-unknown-linux-gnu".into(),
        toolchain: "rustc 1.96.0".into(),
    };
    let mirrors = [
        ("src/a.rs", "tests/a.rs", "mirrors_a", "receipts/a.json"),
        ("src/b.rs", "tests/b.rs", "mirrors_b", "receipts/b.json"),
    ]
    .into_iter()
    .map(|(production, test, name, receipt)| {
        let mirror = TestMirrorContract {
            production: production.into(),
            test: test.into(),
            name: name.into(),
            receipt: receipt.into(),
            inputs: Vec::new(),
            execution: execution.clone(),
            reason: "Exact bulk mirror fixture".into(),
        };
        PlannedTestMirror::new(&mirror, "a".repeat(64)).expect("plan mirror")
    })
    .collect();
    crate::MirrorPlan::new("b".repeat(64), AnalysisMetrics::default(), mirrors)
        .expect("construct plan")
}
