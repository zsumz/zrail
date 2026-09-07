//! Differential key fixtures bind exact authored identities and positive legacy projections.

use super::super::{metadata, model::project};
use super::suite;

#[test]
fn frozen_dependency_key_sets_agree_on_every_identity_and_type_boundary() {
    let suite = suite();
    let root = std::env::temp_dir()
        .canonicalize()
        .expect("canonical temporary directory")
        .join(format!(
            "zrail-key-set-parity-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
    let (policies, _) = metadata::model::policies(&project(), &suite);
    let source = project().join("crates/zrail-testkit/tests/fixtures/rc9/key-sets/valid");
    let inputs = metadata::model::inputs(&source, &policies, &suite);
    assert_eq!(inputs.len(), suite.input_count);
    let outcomes = metadata::fixtures::qualify(&root, &inputs, &policies, &suite);
    assert_eq!(outcomes.len(), suite.fixture_count);
    assert_eq!(
        outcomes.iter().filter(|row| row.native_accepted).count(),
        suite.positive_count
    );
    for policy in &policies {
        let id = format!("repository:file:{}", policy.name);
        let cases = outcomes
            .iter()
            .filter(|row| row.policy_id == id)
            .collect::<Vec<_>>();
        assert!(cases.iter().any(|row| row.native_accepted));
        assert!(
            cases
                .iter()
                .any(|row| row.diagnostic.as_deref() == Some("REP-FILE-007"))
        );
        assert!(
            cases
                .iter()
                .any(|row| row.diagnostic.as_deref() == Some("REP-FILE-006"))
        );
        let names = cases
            .iter()
            .map(|row| &row.case)
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(
            names.len(),
            cases.len(),
            "unique adversarial fixture identities"
        );
    }
}

#[test]
#[ignore = "requires prefetched snapshots and fresh ZRAIL_RC9_KEY_SETS_REPORT"]
fn qualify_all_frozen_kafka_driver_dependency_key_sets() {
    metadata::qualify(&suite());
}
