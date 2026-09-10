//! Frozen expression quantities retain exact suffix/location identity through physical mutations.

use std::collections::{BTreeMap, BTreeSet};

use super::{
    expression_cases, legacy, model,
    mutations::{Mutation, one},
};

pub(super) fn cases(sources: &BTreeMap<String, String>, roots: &[String]) -> Vec<Mutation> {
    let mut cases = vec![Mutation {
        name: "valid".into(),
        changes: BTreeMap::new(),
    }];
    for (index, (source, _)) in expression_cases::CASES.into_iter().enumerate() {
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
    let expected = legacy::expected_associated();
    for (index, key) in expected.keys().enumerate() {
        let (path, suffix) = key.split_once(':').expect("exact file/suffix key");
        let source = &sources[path];
        let removed = source.replacen(suffix, "Rc9Removed::item", 1);
        let before = legacy::associated(path, source);
        let after = legacy::associated(path, &removed);
        assert_eq!(
            before[key],
            after.get(key).copied().unwrap_or(0) + 1,
            "mutation must remove one actual AST occurrence, not matching comment text"
        );
        cases.push(one(
            &format!("remove-{index:02}"),
            path,
            Some(removed.clone()),
        ));
        let addition = format!("\nfn rc9_extra() {{ {suffix}(); }}\n");
        cases.push(one(
            &format!("duplicate-{index:02}"),
            path,
            Some(format!("{source}{addition}")),
        ));
        let other = if suffix.starts_with("ConnectionSet::") {
            "Source::register"
        } else {
            "ConnectionSet::new"
        };
        cases.push(one(
            &format!("same-count-substitution-{index:02}"),
            path,
            Some(source.replacen(suffix, other, 1)),
        ));
        cases.push(Mutation {
            name: format!("same-count-relocation-{index:02}"),
            changes: BTreeMap::from([
                (path.into(), Some(removed.clone())),
                ("src/rc9_moved.rs".into(), Some(addition)),
            ]),
        });
        cases.push(one(
            &format!("call-to-reference-{index:02}"),
            path,
            Some(format!(
                "{removed}\nfn rc9_reference() {{ let _f = {suffix}; }}\n"
            )),
        ));
        cases.push(one(
            &format!("qualified-{index:02}"),
            path,
            Some(source.replacen(suffix, &format!("crate::rc9_qualified::{suffix}"), 1)),
        ));
    }
    for root in roots {
        cases.push(one(
            &format!("new-file-{root}"),
            &format!("{root}/rc9_fresh.rs"),
            Some("fn probe() { ConnectionSet::new(); }".into()),
        ));
    }
    cases.push(one(
        "test-path-exclusion",
        "src/rc9_test.rs",
        Some("fn probe() { Source::register(); }".into()),
    ));
    cases.push(one(
        "parse-malformed",
        "src/rc9_adversary.rs",
        Some("fn bad( {".into()),
    ));
    cases.push(one(
        "parse-expression-fragment",
        "src/rc9_adversary.rs",
        Some("{ ConnectionSet::new(); }".into()),
    ));
    for path in expected
        .keys()
        .map(|key| key.split_once(':').expect("file identity").0)
        .collect::<BTreeSet<_>>()
    {
        cases.push(one(&format!("required-file-deleted-{path}"), path, None));
    }
    cases
}
