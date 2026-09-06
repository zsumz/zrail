//! Every raw dependency assertion reaches its own diagnostic with the frozen detector beside it.

use super::super::{metadata, model::project};
use super::suite;

#[test]
fn frozen_raw_dependency_checks_preserve_line_spelling_counts_and_read_preconditions() {
    let suite = suite();
    let root = std::env::temp_dir().join(format!(
        "zrail-raw-dependency-parity-{}-{:?}",
        std::process::id(),
        std::thread::current().id(),
    ));
    let (policies, _) = metadata::model::policies(&project(), &suite);
    let source = project().join("crates/zrail-testkit/tests/fixtures/rc9/raw-dependency/valid");
    let inputs = metadata::model::inputs(&source, &policies, &suite);
    assert_eq!(inputs.len(), suite.input_count);
    let outcomes = metadata::fixtures::qualify(&root, &inputs, &policies, &suite);
    assert_eq!(outcomes.len(), suite.fixture_count);
    assert_eq!(
        outcomes.iter().filter(|row| row.native_accepted).count(),
        suite.positive_count
    );
    for policy in policies {
        let id = format!("repository:file:{}", policy.name);
        let rows = outcomes
            .iter()
            .filter(|row| row.policy_id == id)
            .collect::<Vec<_>>();
        assert!(rows.iter().any(|row| row.native_accepted));
        assert!(rows.iter().any(|row| !row.native_accepted));
        assert_eq!(
            rows.iter()
                .map(|row| &row.case)
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            rows.len()
        );
    }
}

#[test]
#[ignore = "requires prefetched snapshots and fresh ZRAIL_RC9_RAW_DEPENDENCY_REPORT"]
fn qualify_all_frozen_kafka_driver_raw_dependency_assertions() {
    metadata::qualify(&suite());
}
