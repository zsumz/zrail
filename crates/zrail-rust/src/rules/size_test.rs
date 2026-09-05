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
#[ignore = "requires explicitly prefetched frozen snapshots and a fresh size-report output path"]
fn qualify_all_frozen_kafka_size_instances() {
    qualify::run();
}
