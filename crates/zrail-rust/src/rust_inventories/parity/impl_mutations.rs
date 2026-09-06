//! Exact impl membership mutations keep written trait/type identity separate from quantity.

use std::collections::{BTreeMap, BTreeSet};

use super::{
    impl_cases, model,
    mutations::{Mutation, one},
};

pub(super) fn written_pairs(original: BTreeSet<String>) -> BTreeMap<String, usize> {
    original
        .into_iter()
        .map(|key| {
            let parts = key.split(':').collect::<Vec<_>>();
            assert_eq!(parts.len(), 3, "exact frozen file/type/trait tuple");
            (format!("{}:{} for {}", parts[0], parts[2], parts[1]), 1)
        })
        .collect()
}

pub(super) fn original_pairs(counts: &BTreeMap<String, usize>) -> BTreeSet<String> {
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

pub(super) fn cases(sources: &BTreeMap<String, String>, roots: &[String]) -> Vec<Mutation> {
    let mut cases = vec![Mutation {
        name: "valid".into(),
        changes: BTreeMap::new(),
    }];
    for (index, (source, _)) in impl_cases::CASES.iter().enumerate() {
        cases.push(one(
            &format!("syntax-{index:02}"),
            "src/rc9_adversary.rs",
            Some((*source).into()),
        ));
    }
    cases.push(one(
        "original-detector",
        "src/reactor/rogue.rs",
        Some(model::detector_source()),
    ));
    for root in roots {
        cases.push(one(
            &format!("new-file-{root}"),
            &format!("{root}/rc9_fresh.rs"),
            Some("impl Source for DirectRustlsTransport {}".into()),
        ));
    }
    cases.push(one(
        "test-path-exclusion",
        "src/rc9_test.rs",
        Some("impl Source for DirectRustlsTransport {}".into()),
    ));
    for (name, source) in [
        ("parse-malformed", "impl Source for {"),
        (
            "parse-expression-fragment",
            "{ impl Source for DirectRustlsTransport {} }",
        ),
    ] {
        cases.push(one(name, "src/rc9_adversary.rs", Some(source.into())));
    }
    let owner = "src/reactor/direct_plaintext/rustls_transport.rs";
    let source = &sources[owner];
    cases.push(one("required-file-deleted", owner, None));
    for name in ["RegisteredTransport", "SlotTransport", "Source"] {
        let header = format!("impl {name} for DirectRustlsTransport");
        assert_eq!(
            source.matches(&header).count(),
            1,
            "one reviewed impl header"
        );
        let removed = source.replace(&header, "impl Rc9Removed for DirectRustlsTransport");
        let other = if name == "SlotTransport" {
            "RegisteredTransport"
        } else {
            "SlotTransport"
        };
        for (case, header) in [
            (
                "impl-removed",
                "impl Rc9Removed for DirectRustlsTransport".into(),
            ),
            ("type-exchanged", format!("impl {name} for Rc9Other")),
            (
                "trait-exchanged",
                format!("impl {other} for DirectRustlsTransport"),
            ),
            (
                "qualified",
                format!("impl crate::traits::{name} for crate::types::DirectRustlsTransport"),
            ),
            (
                "raw-trait",
                format!("impl r#{name} for DirectRustlsTransport"),
            ),
            (
                "raw-type",
                format!("impl {name} for r#DirectRustlsTransport"),
            ),
            (
                "negative",
                format!("impl !{name} for DirectRustlsTransport"),
            ),
        ] {
            cases.push(one(
                &format!("{case}-{name}"),
                owner,
                Some(source.replace(&format!("impl {name} for DirectRustlsTransport"), &header)),
            ));
        }
        cases.push(one(
            &format!("duplicate-{name}"),
            owner,
            Some(format!("{source}\n{header} {{}}\n")),
        ));
        cases.push(one(
            &format!("comment-only-{name}"),
            owner,
            Some(format!("{removed}\n// {header} {{}}\n")),
        ));
        cases.push(one(
            &format!("macro-only-{name}"),
            owner,
            Some(format!(
                "{removed}\nmacro_rules! hidden {{ () => {{ {header} {{}} }}; }}\n"
            )),
        ));
        cases.push(Mutation {
            name: format!("moved-{name}"),
            changes: BTreeMap::from([
                (owner.into(), Some(removed)),
                ("src/rc9_moved.rs".into(), Some(format!("{header} {{}}"))),
            ]),
        });
    }
    cases
}
