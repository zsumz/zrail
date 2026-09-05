//! Pinned size-only differential qualification; never a partial repository lock.

#[path = "size/snapshot/legacy_driver.rs"]
mod legacy_driver;
#[path = "size/snapshot/legacy_kafkars.rs"]
mod legacy_kafkars;
#[path = "size/snapshot/model.rs"]
mod model;
#[path = "size/snapshot/qualify.rs"]
mod qualify;

use model::{Budget, BudgetAllow, BudgetBaseline};

#[test]
fn translated_selectors_preserve_future_paths_and_test_facade_precedence() {
    use crate::{
        inventory::FileClass,
        source::{Reachability, ReachabilityKind},
        source_budget,
    };
    let (driver, _) = model::fragment("kafka-driver");
    let (kafkars, _) = model::fragment("kafkars");
    for (fragment, cases) in [
        (
            &driver,
            vec![
                ("tests/new/mod.rs", Some(100)),
                ("tests/guardrails.rs", Some(100)),
                ("src/new_test.rs", Some(320)),
                ("src/nested/tests/ordinary.rs", Some(240)),
                ("src/new.RS", None),
                ("src/not_rust.txt", None),
            ],
        ),
        (
            &kafkars,
            vec![
                ("crates/kafka-client-engine/tests/new/mod.rs", Some(300)),
                (
                    "crates/kafka-client-engine/src/nested/tests/ordinary.rs",
                    Some(240),
                ),
                ("crates/kafka-client-engine/examples/new/mod.rs", Some(300)),
                ("crates/kafka-client-engine/src/new/mod.rs", Some(80)),
                ("crates/kafka-client-engine/src/new_test.rs", Some(300)),
                (
                    "crates/kafka-client-guardrails/tests/fixtures/new/src/a.rs",
                    None,
                ),
                ("crates/kafka-client-engine/src/new.RS", None),
            ],
        ),
    ] {
        for (path, target) in cases {
            let production = Reachability::from_kind(ReachabilityKind::Production);
            for reachability in [
                production,
                Reachability::test(),
                production.join(Reachability::test()),
            ] {
                let budget = source_budget::for_path(
                    path,
                    FileClass::Test,
                    reachability,
                    &[],
                    &fragment.source.rust,
                )
                .expect("noncompeting translated selection");
                assert_eq!(
                    budget.as_ref().map(|b| b.thresholds.target),
                    target,
                    "{path}"
                );
                assert!(
                    budget.is_none_or(|budget| budget.exception.is_none()),
                    "{path}"
                );
            }
        }
    }
}

#[test]
fn frozen_ten_line_facade_fixture_keeps_its_smallest_ceiling() {
    let (mut fragment, _) = model::fragment("kafka-driver");
    let budgets = legacy_driver::Budgets {
        facade: 10,
        production: 20,
        test: 30,
    };
    for scope in &mut fragment
        .source
        .rust
        .budgets
        .as_mut()
        .expect("budgets")
        .overrides
    {
        let value = match scope.name.as_str() {
            "kd-facade" => 10,
            "kd-production" => 20,
            "kd-test" => 30,
            _ => panic!("unexpected frozen size scope"),
        };
        scope.budget.target = value;
        scope.budget.hard = value;
    }
    let root = std::path::Path::new("/workspace");
    for (path, expected) in [
        ("src/lib.rs", 10),
        ("src/ordinary.rs", 20),
        ("tests/ordinary.rs", 30),
    ] {
        assert_eq!(
            legacy_driver::limit(root, &root.join(path), budgets),
            expected
        );
        let native = crate::source_budget::for_path(
            path,
            crate::inventory::FileClass::Test,
            crate::source::Reachability::test(),
            &[],
            &fragment.source.rust,
        )
        .expect("selection")
        .expect("active budget");
        assert_eq!(native.thresholds.target, expected);
        assert_eq!(native.hard_ceiling(), expected);
    }
}

#[test]
#[ignore = "requires explicitly prefetched frozen snapshots and a fresh size-report output path"]
fn qualify_all_frozen_kafka_size_instances() {
    qualify::run();
}

fn git(root: &std::path::Path, args: &[&str]) -> String {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .expect("trusted snapshot Git inspection");
    assert!(output.status.success(), "{:?}", output.stderr);
    String::from_utf8(output.stdout).expect("UTF-8 Git output")
}
