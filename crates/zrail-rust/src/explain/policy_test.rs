//! Agent-facing policy vocabulary and sibling paths remain stable.

use zrail_core::{ExternalDependencyMode, LintSuppressionMode, PolicyMode};

use super::{external_mode, lint_mode, policy_mode, sibling_path};

#[test]
fn policy_modes_use_contract_spellings() {
    assert_eq!(policy_mode(PolicyMode::Deny), "deny");
    assert_eq!(lint_mode(LintSuppressionMode::Reasoned), "reasoned");
    assert_eq!(external_mode(ExternalDependencyMode::Locked), "locked");
}

#[test]
fn sibling_paths_replace_only_rust_source_suffixes() {
    assert_eq!(
        sibling_path("crates/core/src/worker.rs").as_deref(),
        Some("crates/core/src/worker_test.rs")
    );
    assert_eq!(sibling_path("crates/core/src/worker_test.rs"), None);
    assert_eq!(sibling_path("README.md"), None);
}
