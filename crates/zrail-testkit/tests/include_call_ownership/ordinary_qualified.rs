//! Qualified aliases retain policy identity without crossing an include edge.

use zrail_core::{AnalysisQuality, Report};

use super::{
    PRODUCTION_OWNER, assert_no_owner_findings, check, fixture, reset, write, write_executor,
    write_lock,
};

#[test]
fn same_file_self_alias_cannot_bypass_call_ownership() {
    let root = fixture("ordinary-qualified-self", "");
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\nuse std::process::Command as Spawn;\npub fn hidden() { let _ = self::Spawn::new(\"sh\"); }\n",
    );
    write_executor(&root);
    write_lock(&root);

    assert_exact_owned_call(&check(&root), "src/lib.rs");
    reset(&root);
}

#[test]
fn self_ignores_a_block_alias_when_entering_the_module_namespace() {
    let root = fixture("ordinary-qualified-self-block", "");
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\nuse std::process::Command as Spawn;\npub fn hidden() { use std::fs::File as Spawn; let _ = self::Spawn::new(\"sh\"); }\n",
    );
    write_executor(&root);
    write_lock(&root);

    assert_exact_owned_call(&check(&root), "src/lib.rs");
    reset(&root);
}

#[test]
fn self_crosses_an_expression_include_to_the_effective_module() {
    let root = fixture("ordinary-qualified-self-expression", "");
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\nuse std::process::Command as Spawn;\npub fn hidden() { let _ = include!(\"expr.rs\"); }\n",
    );
    write(
        &root,
        "src/expr.rs",
        "{ use std::fs::File as Spawn; let _ = self::Spawn::new(\"sh\"); 0 }\n",
    );
    write_executor(&root);
    write_lock(&root);

    assert_exact_owned_call(&check(&root), "src/expr.rs");
    reset(&root);
}

#[test]
fn crate_root_alias_cannot_bypass_call_ownership() {
    let root = fixture("ordinary-qualified-crate", "");
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\nuse std::process::Command as Spawn;\npub fn hidden() { let _ = crate::Spawn::new(\"sh\"); }\n",
    );
    write_executor(&root);
    write_lock(&root);

    assert_exact_owned_call(&check(&root), "src/lib.rs");
    reset(&root);
}

#[test]
fn external_child_super_alias_cannot_bypass_call_ownership() {
    let root = fixture("ordinary-qualified-super", "");
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\nmod child;\nuse std::process::Command as Spawn;\n",
    );
    write(
        &root,
        "src/child.rs",
        "//! Child.\npub fn hidden() { let _ = super::Spawn::new(\"sh\"); }\n",
    );
    write_executor(&root);
    write_lock(&root);

    assert_exact_owned_call(&check(&root), "src/child.rs");
    reset(&root);
}

#[test]
fn inline_child_self_super_alias_cannot_bypass_call_ownership() {
    let root = fixture("ordinary-qualified-self-super", "");
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\nuse std::process::Command as Spawn;\nmod child { pub fn hidden() { let _ = self::super::Spawn::new(\"sh\"); } }\n",
    );
    write_executor(&root);
    write_lock(&root);

    assert_exact_owned_call(&check(&root), "src/lib.rs");
    reset(&root);
}

#[test]
fn test_only_qualified_alias_does_not_change_production_identity() {
    let root = fixture("ordinary-qualified-test", PRODUCTION_OWNER);
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\n#[cfg(not(test))]\nuse crate::Benign as Spawn;\n#[cfg(test)]\nuse std::process::Command as Spawn;\npub fn hidden() { let _ = self::Spawn::new(\"sh\"); }\npub struct Benign;\n",
    );
    write_executor(&root);
    write_lock(&root);

    let report = check(&root);
    assert_exact_owned_call(&report, "src/lib.rs");
    assert_no_owner_findings(&report, "production-process");
    reset(&root);
}

#[test]
fn qualified_alias_in_an_allowed_owner_remains_exact() {
    let root = fixture("ordinary-qualified-allowed", "");
    write(&root, "src/lib.rs", "//! Library.\nmod executor;\n");
    write(
        &root,
        "src/executor.rs",
        "//! Authorized process owner.\nuse std::process::Command as Spawn;\npub fn allowed() { let _ = self::Spawn::new(\"true\"); }\n",
    );
    write_lock(&root);

    assert_no_owner_findings(&check(&root), "process-spawn");
    reset(&root);
}

fn assert_exact_owned_call(report: &Report, path: &str) {
    assert!(
        report.findings.iter().any(|finding| {
            finding.id == "OWN-003"
                && finding.rule == "process-spawn"
                && finding.path.as_deref() == Some(path)
                && finding.analysis == AnalysisQuality::Exact
        }),
        "{}",
        report.human()
    );
}
