//! Persistent rename bans retain exact source/alias identities and original set deduplication.

use std::collections::BTreeMap;

use super::{
    model,
    mutations::{Mutation, one},
    rename_cases,
};

pub(super) fn cases(sources: &BTreeMap<String, String>, roots: &[String]) -> Vec<Mutation> {
    let mut cases = vec![Mutation {
        name: "valid".into(),
        changes: BTreeMap::new(),
    }];
    for (index, (source, _)) in rename_cases::CASES.iter().enumerate() {
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
            Some("use p::Source as Io;".into()),
        ));
    }
    cases.push(one(
        "test-path-exclusion",
        "src/rc9_test.rs",
        Some("use p::Source as Io;".into()),
    ));
    for (name, source) in [
        ("parse-malformed", "use p::{"),
        ("parse-expression-fragment", "{ use p::Source as Io; }"),
    ] {
        cases.push(one(name, "src/rc9_adversary.rs", Some(source.into())));
    }
    for name in [
        "ConnectionSet",
        "DirectSet",
        "RegisteredTransport",
        "SlotTransport",
        "Source",
    ] {
        for (case, source) in [
            ("new-alias", format!("use p::{name} as Alias;")),
            ("same-name", format!("use p::{name} as {name};")),
            ("underscore", format!("use p::{name} as _;")),
            ("raw-source", format!("use p::r#{name} as Alias;")),
            ("raw-alias", format!("use p::{name} as r#Alias;")),
            (
                "duplicate",
                format!("use p::{name} as Alias; use q::{name} as Alias;"),
            ),
            ("destination-only", format!("use p::Other as {name};")),
            (
                "grouped",
                format!("pub use p::{{nested::{{{name} as Alias}}, *}};"),
            ),
            (
                "function-import",
                format!("fn probe() {{ use p::{name} as Alias; }}"),
            ),
        ] {
            cases.push(one(
                &format!("{case}-{name}"),
                "src/rc9_adversary.rs",
                Some(source),
            ));
        }
    }
    let deleted = "src/reactor/direct_plaintext/set_owner.rs";
    assert!(sources.contains_key(deleted), "frozen selected input");
    cases.push(one("selected-file-deleted", deleted, None));
    cases
}
