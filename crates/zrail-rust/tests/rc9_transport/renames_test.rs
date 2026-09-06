//! The original rename set assertion runs against every typed scope and identity mutation.

use super::{family::Family, fixtures, model, qualification};

#[test]
#[ignore = "requires prefetched snapshots and fresh ZRAIL_RC9_TRANSPORT_RENAMES_REPORT"]
fn qualify_all_frozen_kafka_driver_transport_renames() {
    qualification::run(Family::Renames);
}

#[test]
fn frozen_rename_assertion_preserves_zero_bans_and_exact_alias_identities() {
    let family = Family::Renames;
    let (policy, _) = model::policy_for(family);
    let roots = policy
        .include
        .iter()
        .map(|p| p.strip_suffix("/**/*.rs").expect("root selector").into())
        .collect::<Vec<String>>();
    let sources = std::collections::BTreeMap::from([(
        "src/reactor/direct_plaintext/set_owner.rs".into(),
        "use p::ConnectionSet;".into(),
    )]);
    let root = std::env::temp_dir().join(format!(
        "zrail-rename-fixtures-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let cases = fixtures::qualify_family(&root, &roots, &sources, &policy, family);
    assert_eq!(cases.len(), family.totals().0);
    assert_eq!(
        cases.iter().filter(|row| row.native_accepted).count(),
        family.totals().1
    );
}
