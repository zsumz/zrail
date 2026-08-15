//! Ratchet rule vocabulary remains closed and explicit.

use crate::contract::{
    FacadeMode, GeneratedSourceContract, HygieneContract, LintSuppressionMode, ModuleDocsMode,
    PolicyMode, RustSourceContract, TestMode,
};

use super::{compatible_with_test_mode, file_size_policy_applies, supported_rule};

#[test]
fn inline_tests_are_supported_without_opening_extensible_rule_names() {
    assert!(supported_rule("rust.file-size"));
    assert!(supported_rule("rust.inline-tests"));
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
fn file_size_ratchets_require_an_effective_budget() {
    let mut rust = RustSourceContract {
        module_docs: ModuleDocsMode::Allow,
        facades: FacadeMode::Allow,
        entrypoints: FacadeMode::Allow,
        tests: TestMode::Allow,
        generated: Vec::new(),
        out_dir: Vec::new(),
        item_macros: Vec::new(),
        hygiene: HygieneContract {
            unsafe_code: PolicyMode::Allow,
            lint_suppressions: LintSuppressionMode::Allow,
            deny_methods: Vec::new(),
            deny_macros: Vec::new(),
        },
        size: None,
    };
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
