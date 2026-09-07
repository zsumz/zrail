//! Every reviewed provenance document field has intended positive and negative detector evidence.

use super::super::{metadata, model::project};
use super::suite;

#[test]
fn frozen_provenance_fields_preserve_exact_reference_tables_and_root_override_absence() {
    let suite = suite();
    let root = std::env::temp_dir()
        .canonicalize()
        .expect("canonical temporary directory")
        .join(format!(
            "zrail-provenance-parity-{}-{:?}",
            std::process::id(),
            std::thread::current().id(),
        ));
    let (policies, _) = metadata::model::policies(&project(), &suite);
    let source = project().join("crates/zrail-testkit/tests/fixtures/rc9/provenance/valid");
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
#[ignore = "requires prefetched snapshots and fresh ZRAIL_RC9_PROVENANCE_REPORT"]
fn qualify_all_frozen_kafka_driver_provenance_documents() {
    metadata::qualify(&suite());
}
