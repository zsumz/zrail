//! Qualified-self calls retain their complete direct-call identity.

use zrail_core::{AnalysisQuality, Report};

use super::super::{
    assert_no_owner_findings, check, fixture, reset, write, write_executor, write_lock,
};

#[test]
fn direct_qualified_self_call_outside_the_owner_fails() {
    let root = fixture("qualified-self-direct", "");
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\npub fn hidden() { let _ = <std::process::Command>::new(\"sh\"); }\n",
    );
    write_executor(&root);
    write_lock(&root);

    assert_exact_owned_call(&check(&root), "src/lib.rs");
    reset(&root);
}

#[test]
fn qualified_self_call_through_an_ordinary_alias_fails() {
    let root = fixture("qualified-self-alias", "");
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\nuse std::process::Command as Spawn;\npub fn hidden() { let _ = <Spawn>::new(\"sh\"); }\n",
    );
    write_executor(&root);
    write_lock(&root);

    assert_exact_owned_call(&check(&root), "src/lib.rs");
    reset(&root);
}

#[test]
fn qualified_self_call_through_an_include_alias_fails() {
    let root = fixture("qualified-self-include", "");
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\ninclude!(\"imports.rs\");\npub fn hidden() { let _ = <Spawn>::new(\"sh\"); }\n",
    );
    write(
        &root,
        "src/imports.rs",
        "use std::process::Command as Spawn;\n",
    );
    write_executor(&root);
    write_lock(&root);

    assert_exact_owned_call(&check(&root), "src/lib.rs");
    reset(&root);
}

#[test]
fn qualified_self_call_through_a_module_reexport_fails() {
    let root = fixture("qualified-self-module", "");
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\nmod bridge { pub use std::process::Command as Spawn; }\npub fn hidden() { let _ = <crate::bridge::Spawn>::new(\"sh\"); }\n",
    );
    write_executor(&root);
    write_lock(&root);

    assert_exact_owned_call(&check(&root), "src/lib.rs");
    reset(&root);
}

#[test]
fn qualified_self_call_inside_the_allowed_owner_remains_exact() {
    let root = fixture("qualified-self-allowed", "");
    write(&root, "src/lib.rs", "//! Library.\nmod executor;\n");
    write(
        &root,
        "src/executor.rs",
        "//! Authorized process owner.\nuse std::process::Command;\npub fn allowed() { let _ = <Command>::new(\"true\"); }\n",
    );
    write_lock(&root);

    assert_no_owner_findings(&check(&root), "process-spawn");
    reset(&root);
}

#[test]
fn opaque_qualified_self_type_fails_closed() {
    let root = fixture("qualified-self-opaque", "");
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\ntrait Factory { type Output; }\nstruct Choice;\nimpl Factory for Choice { type Output = std::process::Command; }\ntype Spawn = <Choice as Factory>::Output;\npub fn hidden() { let _ = <Spawn>::new(\"sh\"); }\n",
    );
    write_executor(&root);
    write_lock(&root);

    let report = check(&root);
    assert!(
        report.findings.iter().any(|finding| {
            finding.id == "RUST-INCLUDE-002"
                && finding.path.as_deref() == Some("src/lib.rs")
                && finding.analysis == AnalysisQuality::Unresolved
        }),
        "{}",
        report.human()
    );
    reset(&root);
}

#[test]
fn non_path_qualified_self_type_fails_closed() {
    let root = fixture("qualified-self-non-path", "");
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\npub fn hidden() { let _ = <(std::process::Command)>::new(\"sh\"); }\n",
    );
    write_executor(&root);
    write_lock(&root);

    let report = check(&root);
    assert_unresolved_call_resolution(&report);
    reset(&root);
}

#[test]
fn generic_associated_self_type_fails_closed() {
    let root = fixture("qualified-self-generic", "");
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\npub trait Factory { type Output; }\npub fn hidden<process: Factory<Output = std::process::Command>>() { let _ = <process::Output>::new(\"sh\"); }\n",
    );
    write_executor(&root);
    write_lock(&root);

    let report = check(&root);
    assert_unresolved_call_resolution(&report);
    reset(&root);
}

#[test]
fn generic_associated_self_type_in_expression_include_fails_closed() {
    let root = fixture("qualified-self-generic-include", "");
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\npub trait Factory { type Output; }\npub fn hidden<process: Factory<Output = std::process::Command>>() { let _ = include!(\"expression.rs\"); }\n",
    );
    write(
        &root,
        "src/expression.rs",
        "<process::Output>::new(\"sh\")\n",
    );
    write_executor(&root);
    write_lock(&root);

    let report = check(&root);
    assert!(
        report.findings.iter().any(|finding| {
            finding.id == "RUST-INCLUDE-002"
                && finding.path.as_deref() == Some("src/expression.rs")
                && finding.analysis == AnalysisQuality::Unresolved
        }),
        "{}",
        report.human()
    );
    reset(&root);
}

#[test]
fn impl_self_qualified_type_fails_closed() {
    let root = fixture("qualified-self-impl-self", "");
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\npub trait Launch { fn hidden(); }\nimpl Launch for std::process::Command { fn hidden() { let _ = <Self>::new(\"sh\"); } }\n",
    );
    write_executor(&root);
    write_lock(&root);

    let report = check(&root);
    assert_unresolved_call_resolution(&report);
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

fn assert_unresolved_call_resolution(report: &Report) {
    assert!(
        report.findings.iter().any(|finding| {
            finding.id == "RUST-CALL-001"
                && finding.path.as_deref() == Some("src/lib.rs")
                && finding.analysis == AnalysisQuality::Unresolved
        }),
        "{}",
        report.human()
    );
}
