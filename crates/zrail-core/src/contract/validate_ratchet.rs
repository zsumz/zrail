//! Ratchets support only exact, measurable, tightening repository debt.

use std::collections::BTreeSet;

use super::{
    Contract, LintSuppressionMode, ModuleDocsMode, PolicyMode, RustSourceContract, TestMode,
    validate_limits::ValidationErrors, validate_paths::validate_repository_literal,
    validate_sets::require_reason,
};

pub(super) fn validate(contract: &Contract, errors: &mut ValidationErrors) {
    let mut identities = BTreeSet::new();
    for ratchet in &contract.ratchets {
        if !supported_rule(&ratchet.rule) {
            errors.push(format!("unsupported ratchet rule {:?}", ratchet.rule));
        }
        if !compatible_with_test_mode(&ratchet.rule, contract.source.rust.tests) {
            errors.push(format!(
                "ratchet {} requires source.rust.tests = \"sibling\"",
                ratchet.rule
            ));
        }
        if ratchet.rule == "rust.file-size"
            && !file_size_policy_applies(&contract.source.rust, &ratchet.target)
        {
            errors.push(format!(
                "file-size ratchet for {:?} has no handwritten or generated size policy",
                ratchet.target
            ));
        }
        if !compatible_with_rust_policy(&ratchet.rule, &contract.source.rust) {
            errors.push(format!(
                "ratchet {} requires its corresponding strict Rust source policy",
                ratchet.rule
            ));
        }
        validate_repository_literal(&ratchet.target, errors);
        require_reason("ratchet", &ratchet.target, &ratchet.reason, errors);
        if !identities.insert((&ratchet.rule, &ratchet.target)) {
            errors.push(format!(
                "duplicate ratchet {} for {}",
                ratchet.rule, ratchet.target
            ));
        }
    }
}

fn file_size_policy_applies(rust: &super::RustSourceContract, target: &str) -> bool {
    rust.size.is_some()
        || rust
            .generated
            .iter()
            .any(|generated| under_root(target, &generated.root))
}

fn under_root(path: &str, root: &str) -> bool {
    root == "." || path == root || path.starts_with(&format!("{root}/"))
}

fn supported_rule(rule: &str) -> bool {
    matches!(
        rule,
        "rust.file-size"
            | "rust.inline-tests"
            | "rust.module-docs"
            | "rust.hygiene.unsafe"
            | "rust.hygiene.lint-suppressions"
    )
}

fn compatible_with_test_mode(rule: &str, mode: TestMode) -> bool {
    rule != "rust.inline-tests" || mode == TestMode::Sibling
}

fn compatible_with_rust_policy(rule: &str, rust: &RustSourceContract) -> bool {
    match rule {
        "rust.module-docs" => rust.module_docs == ModuleDocsMode::Required,
        "rust.hygiene.unsafe" => rust.hygiene.unsafe_code == PolicyMode::Deny,
        "rust.hygiene.lint-suppressions" => {
            rust.hygiene.lint_suppressions != LintSuppressionMode::Allow
        }
        _ => true,
    }
}

#[cfg(test)]
#[path = "validate_ratchet_test.rs"]
mod validate_ratchet_test;
