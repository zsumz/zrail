//! Ratchets support only exact, measurable, tightening repository debt.

use std::collections::BTreeSet;

use super::{
    Contract, TestMode, validate_limits::ValidationErrors,
    validate_paths::validate_repository_literal, validate_sets::require_reason,
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

fn supported_rule(rule: &str) -> bool {
    matches!(rule, "rust.file-size" | "rust.inline-tests")
}

fn compatible_with_test_mode(rule: &str, mode: TestMode) -> bool {
    rule != "rust.inline-tests" || mode == TestMode::Sibling
}

#[cfg(test)]
#[path = "validate_ratchet_test.rs"]
mod validate_ratchet_test;
