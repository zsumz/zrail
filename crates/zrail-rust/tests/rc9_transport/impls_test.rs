//! Original trait-implementation detector tests run independently of native quantities.

use std::collections::{BTreeMap, BTreeSet};

use super::{impl_cases, legacy, model, native};

#[test]
fn frozen_impl_pairs_match_every_path_type_and_nested_syntax_context() {
    let root = std::env::temp_dir().join(format!(
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
            original_pairs(&actual),
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
        original_pairs(&actual)
            .iter()
            .map(|key| key.replacen("src/sample.rs:", "src/reactor/rogue.rs:", 1))
            .collect(),
    );
}

fn original_pairs(counts: &BTreeMap<String, usize>) -> BTreeSet<String> {
    counts
        .iter()
        .map(|(key, count)| {
            assert!(*count > 0);
            let (path, identity) = key.split_once(':').expect("physical path");
            let (trait_name, type_name) = identity.split_once(" for ").expect("trait/type pair");
            format!("{path}:{type_name}:{trait_name}")
        })
        .collect()
}
