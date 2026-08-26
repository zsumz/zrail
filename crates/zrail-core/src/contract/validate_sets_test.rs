//! Rust policy selectors use their semantic normalized identity.

use super::{ValidationErrors, rust_selectors};

#[test]
fn raw_identifier_spellings_cannot_duplicate_a_selector() {
    let mut errors = ValidationErrors::new();

    rust_selectors(
        "source.rust.hygiene.deny_methods",
        &["unwrap".into(), "r#unwrap".into()],
        &mut errors,
    );

    assert!(errors.finish().join("\n").contains("duplicate normalized"));
}

#[test]
fn duplicate_async_syntax_is_rejected() {
    let mut contract = crate::contract::validate_fixture_test::minimal_contract();
    contract.profiles.insert(
        "sync".into(),
        crate::ProfileContract {
            reachability: crate::PolicyReachability::All,
            effects: crate::EffectBoundary { deny: Vec::new() },
            syntax: crate::SyntaxBoundary {
                deny: vec![crate::AsyncSyntax::Await, crate::AsyncSyntax::Await],
            },
        },
    );

    let error = crate::contract::validate::validate_contract(&contract)
        .expect_err("duplicate syntax must fail");
    assert!(error.to_string().contains("duplicate syntax Await"));
}
