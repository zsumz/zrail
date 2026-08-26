//! Mirror plans are canonical, digest-bound, and strict before execution.

use zrail_core::{TestExecutionIdentity, TestMirrorContract};

use crate::AnalysisMetrics;

use super::{MirrorPlan, PlannedTestMirror};

#[test]
fn plan_round_trips_and_rejects_payload_drift() {
    let mirror = mirror();
    let entry = PlannedTestMirror::new(&mirror, "a".repeat(64)).expect("planned mirror");
    let plan = MirrorPlan::new("b".repeat(64), AnalysisMetrics::default(), vec![entry])
        .expect("mirror plan");
    let json = plan.json().expect("plan JSON");

    assert_eq!(MirrorPlan::parse(&json).expect("parse plan"), plan);

    let drifted = json.replacen(&"b".repeat(64), &"c".repeat(64), 1);
    assert!(
        MirrorPlan::parse(&drifted)
            .expect_err("digest drift must fail")
            .contains("digest")
    );
}

#[test]
fn delimiter_bearing_paths_round_trip_as_distinct_plan_entries() {
    let mut first = mirror();
    first.production = "src/a.rs".into();
    first.test = "tests/b.rs::c.rs".into();
    first.name = "proof".into();
    let mut second = mirror();
    second.production = "src/a.rs::tests/b.rs".into();
    second.test = "c.rs".into();
    second.name = "proof".into();
    let mut entries = [first, second]
        .iter()
        .map(|mirror| PlannedTestMirror::new(mirror, "a".repeat(64)))
        .collect::<Result<Vec<_>, _>>()
        .expect("planned mirrors");
    entries.sort_by(|left, right| left.policy_id.cmp(&right.policy_id));
    assert_ne!(entries[0].policy_id, entries[1].policy_id);
    let plan = MirrorPlan::new("b".repeat(64), AnalysisMetrics::default(), entries)
        .expect("collision-free plan");

    assert_eq!(
        MirrorPlan::parse(&plan.json().expect("plan JSON")).expect("parse plan"),
        plan,
    );
}

fn mirror() -> TestMirrorContract {
    TestMirrorContract {
        production: "src/state.rs".into(),
        test: "tests/state_test.rs".into(),
        name: "state_transition".into(),
        receipt: "evidence/state.json".into(),
        inputs: vec!["Cargo.lock".into(), "Cargo.toml".into()],
        execution: TestExecutionIdentity {
            command: "cargo test -p fixture state_transition -- --exact".into(),
            package: "fixture".into(),
            default_features: true,
            features: Vec::new(),
            target: "x86_64-unknown-linux-gnu".into(),
            toolchain: "rustc 1.96.0".into(),
        },
        reason: "Exact transition behavior".into(),
    }
}
