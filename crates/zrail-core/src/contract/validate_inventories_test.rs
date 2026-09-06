//! Strict inventory parsing rejects unsupported worlds, hidden selectors, and ambiguous counts.

use crate::{RustInventoryAssertion, RustInventoryRule, RustInventorySubject};

fn rule(assertion: &str) -> RustInventoryRule {
    toml::from_str(&format!(
        "name='methods'\nreason='Reviewed written authority.'\ninclude=['src/**/*.rs']\nworld='authored'\nsubject={{kind='written-methods',names=['poll']}}\nassertion={{{assertion}}}"
    )).expect("inventory schema")
}

fn validate(rule: RustInventoryRule) -> Result<(), crate::ContractError> {
    let mut contract = super::super::validate_fixture_test::minimal_contract();
    contract.source.rust.inventories = vec![rule];
    super::super::validate::validate_contract(&contract)
}

#[test]
fn exact_empty_inventories_and_absent_zero_bans_remain_valid() {
    for assertion in [
        "kind='exact-counts',counts=[]",
        "kind='count',maximum=0",
        "kind='count',minimum=1",
    ] {
        validate(rule(assertion)).expect("nonvacuous requirement");
    }
    let old = super::super::validate_fixture_test::minimal_contract()
        .source
        .rust;
    let serialized = toml::to_string(&old).expect("old source contract");
    assert!(!serialized.contains("inventories"));
    assert!(
        toml::from_str::<crate::RustSourceContract>(&serialized)
            .expect("rc8 default")
            .inventories
            .is_empty()
    );
}

#[test]
fn unknown_syntax_authority_and_wrong_types_fail_strict_parsing() {
    let valid = toml::to_string(&rule("kind='count',maximum=0")).expect("source contract");
    for invalid in [
        valid.replace("authored", "compiled"),
        valid.replace("world = \"authored\"", ""),
        valid.replace("written-methods", "semantic-methods"),
        valid.replace("maximum = 0", "maximum = -1"),
        valid.replace("maximum = 0", "maximum = '0'"),
        valid.replace("maximum = 0", "maximum = 0\nignore_unresolved = true"),
        valid.replace("names =", "receiver = 'Transport'\nnames ="),
    ] {
        assert!(
            toml::from_str::<RustInventoryRule>(&invalid).is_err(),
            "{invalid}"
        );
    }
}

#[test]
fn quantities_are_bounded_and_exact_pairs_must_belong_to_their_selection() {
    for assertion in [
        "kind='count'",
        "kind='count',minimum=2,maximum=1",
        "kind='count',maximum=50001",
        "kind='exact-counts',counts=[{path='src/lib.rs',name='poll',count=0}]",
        "kind='exact-counts',counts=[{path='src/lib.rs',name='other',count=1}]",
        "kind='exact-counts',counts=[{path='other/lib.rs',name='poll',count=1}]",
        "kind='exact-counts',counts=[{path='src/../lib.rs',name='poll',count=1}]",
        "kind='exact-counts',counts=[{path='src/lib.rs',name='poll',count=1},{path='src/lib.rs',name='poll',count=2}]",
    ] {
        assert!(validate(rule(assertion)).is_err(), "{assertion}");
    }
    let mut excluded = rule("kind='exact-counts',counts=[{path='src/lib.rs',name='poll',count=1}]");
    excluded.exclude.push("src/lib.rs".into());
    assert!(validate(excluded).is_err());
    for name in ["", "*", "Type::poll", "self.poll", "_", "poll()"] {
        let mut invalid = rule("kind='count',maximum=0");
        invalid.subject = RustInventorySubject::WrittenMethods {
            names: vec![name.into()],
        };
        assert!(validate(invalid).is_err(), "{name}");
    }
    let mut invalid = rule("kind='count',maximum=0");
    invalid.assertion = RustInventoryAssertion::ExactCounts { counts: vec![] };
    invalid.include.push("src/**/*.rs".into());
    assert!(validate(invalid).is_err());
}

#[test]
fn expression_path_subjects_require_two_exact_written_segments() {
    for suffix in ["State::poll", "r#State::r#poll", "_State::poll2"] {
        let mut selected = rule("kind='count',maximum=0");
        selected.subject = RustInventorySubject::WrittenExpressionPaths {
            suffixes: vec![suffix.into()],
        };
        validate(selected).expect("bounded written suffix");
    }
    for suffix in [
        "poll",
        "*::poll",
        "::State::poll",
        "crate::State::poll",
        "State :: poll",
        "State::poll()",
        "State<T>::poll",
        "_::poll",
        "State::",
    ] {
        let mut selected = rule("kind='count',maximum=0");
        selected.subject = RustInventorySubject::WrittenExpressionPaths {
            suffixes: vec![suffix.into()],
        };
        assert!(validate(selected).is_err(), "{suffix}");
    }
    let mut selected =
        rule("kind='exact-counts',counts=[{path='src/lib.rs',name='State::poll',count=1}]");
    selected.subject = RustInventorySubject::WrittenExpressionPaths {
        suffixes: vec!["State::poll".into()],
    };
    validate(selected.clone()).expect("exact path quantity");
    selected.subject = RustInventorySubject::WrittenExpressionPaths {
        suffixes: vec!["State::poll".into(); 2],
    };
    assert!(validate(selected).is_err());
}
