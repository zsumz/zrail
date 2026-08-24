//! Ratchet rule vocabulary remains closed and explicit.

use crate::contract::{
    FacadeMode, GeneratedSourceContract, HygieneContract, LintSuppressionMode, ModuleDocsMode,
    PolicyMode, RatchetContract, RustSourceContract, TestMode,
    validate_fixture_test::minimal_contract,
};

use super::{
    ValidationErrors, compatible_with_rust_policy, compatible_with_test_mode,
    file_size_policy_applies, supported_rule, validate,
};

#[test]
fn inline_tests_are_supported_without_opening_extensible_rule_names() {
    assert!(supported_rule("rust.file-size"));
    assert!(supported_rule("rust.inline-tests"));
    assert!(supported_rule("rust.module-docs"));
    assert!(supported_rule("rust.hygiene.unsafe"));
    assert!(supported_rule("rust.hygiene.lint-suppressions"));
    assert!(supported_rule("rust.hygiene.denied-method"));
    assert!(supported_rule("rust.hygiene.denied-macro"));
    assert!(!supported_rule("rust.any-future-debt"));
    assert!(compatible_with_test_mode(
        "rust.inline-tests",
        TestMode::Sibling
    ));
    assert!(!compatible_with_test_mode(
        "rust.inline-tests",
        TestMode::Allow
    ));
}

#[test]
fn adoption_ratchets_require_their_strict_policy() {
    let mut rust = rust_contract();
    for rule in [
        "rust.module-docs",
        "rust.hygiene.unsafe",
        "rust.hygiene.lint-suppressions",
    ] {
        assert!(!compatible_with_rust_policy(rule, &rust));
    }

    rust.module_docs = ModuleDocsMode::Required;
    rust.hygiene.unsafe_code = PolicyMode::Deny;
    rust.hygiene.lint_suppressions = LintSuppressionMode::Reasoned;
    for rule in [
        "rust.module-docs",
        "rust.hygiene.unsafe",
        "rust.hygiene.lint-suppressions",
    ] {
        assert!(compatible_with_rust_policy(rule, &rust));
    }
}

#[test]
fn file_size_ratchets_require_an_effective_budget() {
    let mut rust = rust_contract();
    assert!(!file_size_policy_applies(&rust, "src/lib.rs"));

    rust.generated.push(GeneratedSourceContract {
        root: "src/generated".into(),
        manifest: "src/generated/MANIFEST.json".into(),
        inputs: vec!["schema/**".into()],
        target: 800,
        hard: 1_000,
        reason: "compiler output".into(),
        auxiliary: Vec::new(),
    });
    assert!(file_size_policy_applies(&rust, "src/generated/model.rs"));
    assert!(!file_size_policy_applies(&rust, "src/lib.rs"));
}

#[test]
fn denied_operation_ratchets_require_a_configured_selector() {
    let mut contract = minimal_contract();
    contract.source.rust.hygiene.deny_methods = vec!["unwrap".into()];
    contract.ratchets = vec![ratchet("rust.hygiene.denied-method", None)];
    assert!(errors(&contract).contains("requires an exact denied-operation selector"));

    contract.ratchets[0].selector = Some("expect".into());
    assert!(errors(&contract).contains("is not configured"));

    contract.ratchets[0].selector = Some("r#unwrap".into());
    assert!(errors(&contract).is_empty());
}

#[test]
fn selector_identity_is_normalized_and_reserved_for_selector_rules() {
    let mut contract = minimal_contract();
    contract.source.rust.hygiene.deny_methods = vec!["unwrap".into()];
    contract.ratchets = vec![
        ratchet("rust.hygiene.denied-method", Some("unwrap")),
        ratchet("rust.hygiene.denied-method", Some("r#unwrap")),
    ];
    assert!(errors(&contract).contains("duplicate ratchet"));

    contract.ratchets = vec![ratchet("rust.file-size", Some("unwrap"))];
    contract.source.rust.size = Some(crate::FileSizeContract {
        facade: crate::Budget {
            target: 10,
            hard: 20,
        },
        implementation: crate::Budget {
            target: 10,
            hard: 20,
        },
        test: crate::Budget {
            target: 10,
            hard: 20,
        },
        auxiliary: crate::Budget {
            target: 10,
            hard: 20,
        },
    });
    assert!(errors(&contract).contains("does not support a selector"));
}

fn ratchet(rule: &str, selector: Option<&str>) -> RatchetContract {
    RatchetContract {
        rule: rule.into(),
        selector: selector.map(str::to_owned),
        target: "src/lib.rs".into(),
        reason: "legacy debt".into(),
    }
}

fn errors(contract: &crate::Contract) -> String {
    let mut errors = ValidationErrors::new();
    validate(contract, &mut errors);
    errors.finish().join("\n")
}

fn rust_contract() -> RustSourceContract {
    RustSourceContract {
        module_docs: ModuleDocsMode::Allow,
        facades: FacadeMode::Allow,
        entrypoints: FacadeMode::Allow,
        tests: TestMode::Allow,
        file_roles: Vec::new(),
        generated: Vec::new(),
        out_dir: Vec::new(),
        item_macros: Vec::new(),
        macros: crate::MacroExpansionContract::default(),
        hygiene: HygieneContract {
            unsafe_code: PolicyMode::Allow,
            lint_suppressions: LintSuppressionMode::Allow,
            deny_methods: Vec::new(),
            deny_macros: Vec::new(),
        },
        size: None,
    }
}
