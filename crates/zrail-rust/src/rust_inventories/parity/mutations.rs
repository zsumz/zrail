//! Trusted physical counterexamples keep the frozen expected quantity map fixed.

use std::collections::BTreeMap;

use super::{legacy, model, syntax_cases};

pub(super) struct Mutation {
    pub(super) name: String,
    pub(super) changes: BTreeMap<String, Option<String>>,
}

pub(super) fn cases(sources: &BTreeMap<String, String>, roots: &[String]) -> Vec<Mutation> {
    let mut cases = vec![Mutation {
        name: "valid".into(),
        changes: BTreeMap::new(),
    }];
    for (index, source) in syntax_cases::CASES.into_iter().enumerate() {
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
    for (index, (key, _)) in legacy::expected().into_iter().enumerate() {
        let (path, method) = key.rsplit_once(':').expect("exact file/method key");
        let source = &sources[path];
        let written = format!(".{method}()");
        assert!(source.contains(&written), "frozen call spelling");
        let removed = source.replacen(&written, ".rc9_removed()", 1);
        let before = legacy::observed(path, source);
        let after = legacy::observed(path, &removed);
        assert_eq!(
            before[&key],
            after.get(&key).copied().unwrap_or(0) + 1,
            "mutation must remove an actual AST occurrence, not matching comment text"
        );
        cases.push(one(
            &format!("remove-{index:02}"),
            path,
            Some(removed.clone()),
        ));
        let addition = format!("\nfn rc9_extra(value: &()) {{ value.{method}(); }}\n");
        cases.push(one(
            &format!("duplicate-{index:02}"),
            path,
            Some(format!("{source}{addition}")),
        ));
        let other = if method == "wake_handle" {
            "pulse_handle"
        } else {
            "wake_handle"
        };
        cases.push(one(
            &format!("same-count-substitution-{index:02}"),
            path,
            Some(source.replacen(&written, &format!(".{other}()"), 1)),
        ));
        cases.push(Mutation {
            name: format!("same-count-relocation-{index:02}"),
            changes: BTreeMap::from([
                (path.into(), Some(removed)),
                ("src/rc9_moved.rs".into(), Some(addition)),
            ]),
        });
    }
    for root in roots {
        cases.push(one(
            &format!("new-file-{root}"),
            &format!("{root}/rc9_fresh.rs"),
            Some("fn probe(x: X) { x.poll_io(); }".into()),
        ));
    }
    cases.push(one(
        "test-path-exclusion",
        "src/rc9_test.rs",
        Some("fn probe(x: X) { x.poll_io(); }".into()),
    ));
    cases.push(one(
        "parse-malformed",
        "src/rc9_adversary.rs",
        Some("fn bad( {".into()),
    ));
    cases.push(one(
        "parse-expression-fragment",
        "src/rc9_adversary.rs",
        Some("{ value.poll_io(); }".into()),
    ));
    cases.push(one("required-file-deleted", "src/reactor/backend.rs", None));
    cases
}

fn one(name: &str, path: &str, source: Option<String>) -> Mutation {
    Mutation {
        name: name.into(),
        changes: BTreeMap::from([(path.into(), source)]),
    }
}
