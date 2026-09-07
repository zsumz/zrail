//! Original trait-implementation detector tests run independently of native quantities.

use std::collections::BTreeMap;

use super::{
    family::Family, fixtures, impl_cases, impl_mutations, legacy, model, native, qualification,
};

#[test]
fn frozen_impl_pairs_match_every_path_type_and_nested_syntax_context() {
    let root = std::env::temp_dir()
        .canonicalize()
        .expect("canonical temporary directory")
        .join(format!(
            "zrail-impl-parity-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
    for (source, expected) in impl_cases::CASES {
        let expected = expected
            .iter()
            .map(|(identity, count)| (format!("src/sample.rs:{identity}"), *count))
            .collect::<BTreeMap<_, _>>();
        let actual = native::observed_with_subject(&root, source, impl_cases::SUBJECT);
        assert_eq!(actual, expected, "{source}");
        assert_eq!(
            impl_mutations::original_pairs(&actual),
            legacy::api::impls("src/sample.rs", source),
            "{source}"
        );
    }
    let source = model::detector_source();
    legacy::api::check_detector_impls(legacy::api::impls("src/reactor/rogue.rs", &source));
    let actual = native::observed_with_subject(&root, &source, impl_cases::SUBJECT);
    assert_eq!(actual.len(), 1);
    assert_eq!(actual.values().sum::<usize>(), 1);
    legacy::api::check_detector_impls(
        impl_mutations::original_pairs(&actual)
            .iter()
            .map(|key| key.replacen("src/sample.rs:", "src/reactor/rogue.rs:", 1))
            .collect(),
    );
}

#[test]
#[ignore = "requires prefetched snapshots and fresh ZRAIL_RC9_TRANSPORT_IMPLS_REPORT"]
fn qualify_all_frozen_kafka_driver_transport_impls() {
    qualification::run(Family::Impls);
}

#[test]
fn frozen_impl_assertion_preserves_exact_trait_type_and_file_membership() {
    use std::fmt::Write as _;
    let family = Family::Impls;
    let (policy, _) = model::policy_for(family);
    let roots = policy
        .include
        .iter()
        .map(|p| p.strip_suffix("/**/*.rs").expect("root").into())
        .collect::<Vec<String>>();
    let mut source = String::new();
    for name in ["RegisteredTransport", "SlotTransport", "Source"] {
        writeln!(source, "impl {name} for DirectRustlsTransport {{}}").expect("synthetic impl");
    }
    let sources = BTreeMap::from([(
        "src/reactor/direct_plaintext/rustls_transport.rs".into(),
        source,
    )]);
    let root = std::env::temp_dir()
        .canonicalize()
        .expect("canonical temporary directory")
        .join(format!(
            "zrail-impl-fixtures-{}-{:?}",
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
