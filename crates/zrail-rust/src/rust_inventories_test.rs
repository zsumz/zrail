//! Frozen collector parity checks syntax corners before downstream replacement is claimed.

#[path = "rust_inventories/parity/expression_cases.rs"]
mod expression_cases;
#[path = "rust_inventories/parity/legacy.rs"]
mod legacy;
#[path = "rust_inventories/parity/native.rs"]
mod native;
#[path = "rust_inventories/parity/path_cases.rs"]
mod path_cases;
#[path = "rust_inventories/parity/rename_cases.rs"]
mod rename_cases;
#[path = "rust_inventories/parity/selection.rs"]
mod selection;
#[path = "rust_inventories/parity/syntax_cases.rs"]
mod syntax_cases;

#[path = "rust_inventories/parity/expression_mutations.rs"]
mod expression_mutations;
#[path = "rust_inventories/parity/family.rs"]
mod family;
#[path = "../tests/rc9_declarations/file_items.rs"]
mod file_items_test;
#[path = "rust_inventories/parity/fixtures.rs"]
mod fixtures;
#[path = "rust_inventories/parity/impl_cases.rs"]
mod impl_cases;
#[path = "rust_inventories/parity/impl_mutations.rs"]
mod impl_mutations;
#[path = "../tests/rc9_transport/impls_test.rs"]
mod impls_test;
#[path = "rust_inventories/parity/model.rs"]
mod model;
#[path = "rust_inventories/parity/mutations.rs"]
mod mutations;
#[path = "rust_inventories/parity/owner_mutations.rs"]
mod owner_mutations;
#[path = "../tests/rc9_transport/qualification.rs"]
mod qualification;
#[path = "rust_inventories/parity/rename_mutations.rs"]
mod rename_mutations;
#[path = "../tests/rc9_transport/renames_test.rs"]
mod renames_test;

#[test]
fn frozen_transport_methods_match_every_parsed_expression_context() {
    assert_eq!(legacy::expected().len(), 11);
    assert_eq!(legacy::expected().values().sum::<usize>(), 20);
    let root = std::env::temp_dir().join(format!(
        "zrail-method-parity-{}-{:?}",
        std::process::id(),
        std::thread::current().id(),
    ));
    for (index, source) in syntax_cases::CASES.into_iter().enumerate() {
        let expected = legacy::observed("src/sample.rs", source);
        let observed = native::observed(&root, source);
        assert_eq!(observed, expected, "case {index}: {source}");
    }
}

#[test]
fn frozen_transport_assertion_rejects_each_quantity_and_scope_regression() {
    use std::fmt::Write as _;

    let (policy, _) = model::policy();
    let roots = policy
        .include
        .iter()
        .map(|pattern| {
            pattern
                .strip_suffix("/**/*.rs")
                .expect("exact frozen root selector")
                .into()
        })
        .collect::<Vec<String>>();
    let mut sources = std::collections::BTreeMap::<String, String>::new();
    for (key, count) in legacy::expected() {
        let (path, method) = key.rsplit_once(':').expect("reviewed pair");
        let source = sources.entry(path.into()).or_default();
        for index in 0..count {
            writeln!(source, "fn site_{method}_{index}(x: X) {{ x.{method}(); }}")
                .expect("write synthetic source");
        }
    }
    let root = std::env::temp_dir().join(format!(
        "zrail-transport-fixtures-{}-{:?}",
        std::process::id(),
        std::thread::current().id(),
    ));
    let cases = fixtures::qualify(&root, &roots, &sources, &policy);
    assert_eq!(cases.len(), 76);
    assert_eq!(cases.iter().filter(|row| row.native_accepted).count(), 7);
    let detector = legacy::observed("src/reactor/rogue.rs", &model::detector_source());
    legacy::check_detector_methods(detector);
}

#[test]
#[ignore = "requires prefetched snapshots and fresh ZRAIL_RC9_TRANSPORT_METHODS_REPORT"]
fn qualify_all_frozen_kafka_driver_transport_methods() {
    qualification::run(family::Family::Methods);
}

#[test]
#[ignore = "requires prefetched snapshots and fresh ZRAIL_RC9_TRANSPORT_EXPRESSION_PATHS_REPORT"]
fn qualify_all_frozen_kafka_driver_transport_expression_paths() {
    qualification::run(family::Family::ExpressionPaths);
}

#[test]
#[ignore = "requires prefetched snapshots and fresh ZRAIL_RC9_TRANSPORT_OWNERS_REPORT"]
fn qualify_all_frozen_kafka_driver_transport_owners() {
    qualification::run(family::Family::Owners);
}

#[test]
fn frozen_owner_assertion_rejects_missing_and_unexpected_members() {
    let family = family::Family::Owners;
    let (policy, _) = model::policy_for(family);
    let roots = policy
        .include
        .iter()
        .map(|p| p.strip_suffix("/**/*.rs").expect("root selector").into())
        .collect::<Vec<String>>();
    let sources = std::collections::BTreeMap::from([(
        "src/reactor/direct_plaintext/set_owner.rs".into(),
        "fn run(x: ConnectionSet) {}".into(),
    )]);
    let root = std::env::temp_dir().join(format!(
        "zrail-owner-fixtures-{}-{:?}",
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

#[test]
fn frozen_transport_expression_paths_preserve_authored_contexts_and_quantities() {
    let root = std::env::temp_dir().join(format!(
        "zrail-expression-parity-{}-{:?}",
        std::process::id(),
        std::thread::current().id(),
    ));
    for (source, count) in expression_cases::CASES {
        let expected = legacy::associated("src/sample.rs", source);
        assert_eq!(
            expected.values().sum::<usize>(),
            count,
            "frozen semantics: {source}"
        );
        assert_eq!(
            native::observed_with_subject(&root, source, expression_cases::SUBJECT),
            expected,
            "{source}"
        );
    }
}

#[test]
fn frozen_transport_path_membership_preserves_every_written_owner_context() {
    let root = std::env::temp_dir().join(format!(
        "zrail-path-parity-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    for (source, count) in path_cases::CASES {
        let expected = legacy::owners("src/sample.rs", source);
        assert_eq!(
            !expected.is_empty(),
            count > 0,
            "frozen semantics: {source}"
        );
        let actual = native::observed_with_subject(&root, source, path_cases::SUBJECT);
        assert_eq!(actual.values().sum::<usize>(), count, "{source}");
        let owners = actual
            .keys()
            .map(|key| key.split_once(':').expect("owner").0.to_owned())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(owners, expected, "{source}");
    }
    legacy::check_owners(legacy::owners(
        "src/reactor/direct_plaintext/set_owner.rs",
        "fn f(x: ConnectionSet) {}",
    ));
    let detector = model::detector_source();
    legacy::check_detector_owners(legacy::owners("src/reactor/rogue.rs", &detector));
    assert_eq!(
        native::observed_with_subject(&root, &detector, path_cases::SUBJECT),
        std::collections::BTreeMap::from([("src/sample.rs:ConnectionSet".into(), 1)])
    );
}

#[test]
fn frozen_import_renames_preserve_complete_source_and_alias_identity() {
    let root = std::env::temp_dir().join(format!(
        "zrail-rename-parity-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    for (source, counts) in rename_cases::CASES {
        let expected = counts
            .iter()
            .map(|(name, count)| (format!("src/sample.rs:{name}"), *count))
            .collect::<std::collections::BTreeMap<_, _>>();
        let legacy = legacy::api::renames("src/sample.rs", source);
        let actual = native::observed_with_subject(&root, source, rename_cases::SUBJECT);
        assert_eq!(actual, expected, "{source}");
        assert_eq!(
            actual
                .keys()
                .cloned()
                .collect::<std::collections::BTreeSet<_>>(),
            legacy,
            "{source}"
        );
    }
    legacy::api::check_renames(legacy::api::renames("src/sample.rs", "use p::Source;"));
    let detector = model::detector_source();
    legacy::api::check_detector_renames(legacy::api::renames("src/reactor/rogue.rs", &detector));
    let actual = native::observed_with_subject(&root, &detector, rename_cases::SUBJECT);
    assert_eq!(actual.len(), 5);
    assert!(actual.values().all(|count| *count == 1));
    let renamed = actual
        .keys()
        .map(|key| key.replacen("src/sample.rs:", "src/reactor/rogue.rs:", 1))
        .collect();
    legacy::api::check_detector_renames(renamed);
}

#[test]
fn frozen_expression_assertion_rejects_quantity_and_scope_changes() {
    use std::fmt::Write as _;
    let family = family::Family::ExpressionPaths;
    let (policy, _) = model::policy_for(family);
    let roots = policy
        .include
        .iter()
        .map(|p| p.strip_suffix("/**/*.rs").expect("root selector").into())
        .collect::<Vec<String>>();
    let mut sources = std::collections::BTreeMap::<String, String>::new();
    for (key, count) in family.expected() {
        let (path, suffix) = key.split_once(':').expect("exact file/suffix");
        for index in 0..count {
            writeln!(
                sources.entry(path.into()).or_default(),
                "fn site_{}_{}() {{ {suffix}(); }}",
                suffix.replace("::", "_"),
                index
            )
            .expect("fixture source");
        }
    }
    let root = std::env::temp_dir().join(format!(
        "zrail-expression-fixtures-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let cases = fixtures::qualify_family(&root, &roots, &sources, &policy, family);
    assert_eq!(cases.len(), family.totals().0);
    assert_eq!(
        cases.iter().filter(|row| row.native_accepted).count(),
        family.totals().1
    );
    family.check_detector(family.observed("src/reactor/rogue.rs", &model::detector_source()));
}
