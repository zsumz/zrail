//! Distinct owner sets reject missing and relocated members while preserving quantity independence.

use std::collections::BTreeMap;

use super::{
    legacy, model,
    mutations::{Mutation, one},
    path_cases,
};

pub(super) fn cases(sources: &BTreeMap<String, String>, roots: &[String]) -> Vec<Mutation> {
    let mut cases = vec![Mutation {
        name: "valid".into(),
        changes: BTreeMap::new(),
    }];
    for (index, (source, _)) in path_cases::CASES.into_iter().enumerate() {
        cases.push(one(
            &format!("syntax-{index:02}"),
            "src/rc9_adversary.rs",
            Some(source.into()),
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
            Some("fn probe(x: ConnectionSet) {}".into()),
        ));
    }
    cases.push(one(
        "test-path-exclusion",
        "src/rc9_test.rs",
        Some("fn probe(x: ConnectionSet) {}".into()),
    ));
    for (name, source) in [
        ("parse-malformed", "fn bad( {"),
        ("parse-expression-fragment", "{ ConnectionSet::new(); }"),
    ] {
        cases.push(one(name, "src/rc9_adversary.rs", Some(source.into())));
    }
    let owner = legacy::api::expected_owner_files()
        .into_iter()
        .next()
        .expect("one frozen owner");
    let source = &sources[&owner];
    assert!(!legacy::owners(&owner, source).is_empty());
    let removed = source.replace("ConnectionSet", "Rc9Removed");
    assert!(
        legacy::owners(&owner, &removed).is_empty(),
        "remove every original path occurrence"
    );
    cases.push(one("required-file-deleted", &owner, None));
    cases.push(one("owner-removed", &owner, Some(removed.clone())));
    cases.push(Mutation {
        name: "owner-moved".into(),
        changes: BTreeMap::from([
            (owner.clone(), Some(removed)),
            (
                "src/rc9_moved.rs".into(),
                Some("fn probe(x: ConnectionSet) {}".into()),
            ),
        ]),
    });
    cases.push(one(
        "owner-duplicated",
        &owner,
        Some(format!("{source}\nfn extra(x: ConnectionSet) {{}}\n")),
    ));
    for (name, source) in [
        (
            "owner-comment-only",
            "// ConnectionSet\nfn probe() { let _s = \"ConnectionSet\"; stringify!(ConnectionSet); }",
        ),
        ("owner-raw-only", "fn probe(x: r#ConnectionSet) {}"),
        (
            "owner-qualified-only",
            "fn probe(x: ::external::ConnectionSet) {}",
        ),
        (
            "owner-reference-only",
            "fn probe() { let _f = ConnectionSet::new; }",
        ),
        ("owner-types-only", "struct Probe(ConnectionSet);"),
    ] {
        cases.push(one(name, &owner, Some(source.into())));
    }
    cases
}
