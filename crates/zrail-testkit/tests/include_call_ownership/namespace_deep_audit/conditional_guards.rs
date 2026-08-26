//! Conditional bindings retain every possible identity and enforce conservatively.

use zrail_core::{AnalysisQuality, Report};

use super::super::{check, fixture, reset, write, write_executor, write_lock};

#[test]
fn conditional_included_alias_cannot_bypass_call_ownership() {
    let root = fixture("conditional-included-alias", CONDITIONAL_OWNER);
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\npub struct Benign;\nimpl Benign { pub fn new(_: &str) -> Self { Self } }\ninclude!(\"bindings.rs\");\npub fn hidden() { let _ = Spawn::new(\"sh\"); }\n",
    );
    write(
        &root,
        "src/bindings.rs",
        "#[cfg(unix)]\nuse std::process::Command as Spawn;\n#[cfg(not(unix))]\nuse crate::Benign as Spawn;\npub fn allowed() { let _ = std::process::Command::new(\"true\"); }\n",
    );
    write_executor(&root);
    write_lock(&root);

    let report = check(&root);
    assert_owned_call_quality(
        &report,
        "conditional-process",
        "src/lib.rs",
        AnalysisQuality::Conservative,
    );
    assert!(
        !report.findings.iter().any(|finding| {
            matches!(finding.id.as_str(), "OWN-004" | "OWN-005")
                && finding.rule == "conditional-process"
        }),
        "{}",
        report.human()
    );
    reset(&root);
}

#[test]
fn conditional_same_file_alias_cannot_bypass_call_ownership() {
    let root = fixture("conditional-same-file-alias", "");
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\npub struct Benign;\nimpl Benign { pub fn new(_: &str) -> Self { Self } }\n#[cfg(unix)]\nuse std::process::Command as Spawn;\n#[cfg(not(unix))]\nuse crate::Benign as Spawn;\npub fn hidden() { let _ = Spawn::new(\"sh\"); }\n",
    );
    write_executor(&root);
    write_lock(&root);

    assert_unresolved_owned_call(&check(&root), "process-spawn", "src/lib.rs");
    reset(&root);
}

#[test]
fn conditional_glob_cannot_hide_an_owned_call() {
    let root = fixture("conditional-glob", "");
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\nmod benign { pub struct Command; impl Command { pub fn new(_: &str) -> Self { Self } } }\n#[cfg(unix)]\nuse std::process::*;\n#[cfg(not(unix))]\nuse crate::benign::*;\npub fn hidden() { let _ = Command::new(\"sh\"); }\n",
    );
    write_executor(&root);
    write_lock(&root);

    assert_unresolved_owned_call(&check(&root), "process-spawn", "src/lib.rs");
    reset(&root);
}

#[test]
fn conditional_type_alias_cannot_hide_an_owned_call() {
    let root = fixture("conditional-type-alias", "");
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\npub struct Benign;\nimpl Benign { pub fn new(_: &str) -> Self { Self } }\n#[cfg(unix)]\ntype Spawn = std::process::Command;\n#[cfg(not(unix))]\ntype Spawn = Benign;\npub fn hidden() { let _ = Spawn::new(\"sh\"); }\n",
    );
    write_executor(&root);
    write_lock(&root);

    assert_owned_call_quality(
        &check(&root),
        "process-spawn",
        "src/lib.rs",
        AnalysisQuality::Conservative,
    );
    reset(&root);
}

#[test]
fn cfg_attr_binding_fails_closed() {
    let root = fixture("conditional-cfg-attr", "");
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\n#[cfg_attr(unix, allow(unused_imports))]\nuse std::process::Command as Spawn;\npub fn hidden() { let _ = Spawn::new(\"sh\"); }\n",
    );
    write_executor(&root);
    write_lock(&root);

    assert_owned_call_quality(
        &check(&root),
        "process-spawn",
        "src/lib.rs",
        AnalysisQuality::Exact,
    );
    reset(&root);
}

#[test]
fn conditional_call_fact_is_not_dropped_from_active_instances() {
    let root = fixture("conditional-call-fact", "");
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\npub struct Benign;\nimpl Benign { pub fn new(_: &str) -> Self { Self } }\n#[cfg(unix)]\ninclude!(\"bindings.rs\");\n#[cfg(not(unix))]\nuse crate::Benign as Spawn;\n#[cfg(unix)]\npub fn hidden() { let _ = Spawn::new(\"sh\"); }\n",
    );
    write(
        &root,
        "src/bindings.rs",
        "use std::process::Command as Spawn;\n",
    );
    write_executor(&root);
    write_lock(&root);

    assert_owned_call(&check(&root), "process-spawn", "src/lib.rs");
    reset(&root);
}

#[test]
fn conditional_alias_in_an_allowed_owner_requires_exact_resolution() {
    let root = fixture("conditional-allowed-owner", "");
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\npub struct Benign;\nimpl Benign { pub fn new(_: &str) -> Self { Self } }\n",
    );
    write(
        &root,
        "src/executor.rs",
        "//! Authorized process owner.\n#[cfg(unix)]\nuse std::process::Command as Spawn;\n#[cfg(not(unix))]\nuse crate::Benign as Spawn;\npub fn allowed() { let _ = Spawn::new(\"true\"); }\n",
    );
    write_lock(&root);

    let report = check(&root);
    assert!(
        report.findings.iter().any(|finding| {
            finding.id == "OWN-005"
                && finding.rule == "process-spawn"
                && finding.path.as_deref() == Some("src/executor.rs")
                && finding.analysis == AnalysisQuality::Unresolved
        }),
        "{}",
        report.human()
    );
    reset(&root);
}

fn assert_unresolved_owned_call(report: &Report, rule: &str, path: &str) {
    assert!(
        report.findings.iter().any(|finding| {
            finding.id == "OWN-003"
                && finding.rule == rule
                && finding.path.as_deref() == Some(path)
                && finding.analysis == AnalysisQuality::Unresolved
        }),
        "{}",
        report.human()
    );
}

fn assert_owned_call_quality(report: &Report, rule: &str, path: &str, quality: AnalysisQuality) {
    assert!(
        report.findings.iter().any(|finding| {
            finding.id == "OWN-003"
                && finding.rule == rule
                && finding.path.as_deref() == Some(path)
                && finding.analysis == quality
        }),
        "{}",
        report.human()
    );
}

fn assert_owned_call(report: &Report, rule: &str, path: &str) {
    assert!(
        report.findings.iter().any(|finding| {
            finding.id == "OWN-003" && finding.rule == rule && finding.path.as_deref() == Some(path)
        }),
        "{}",
        report.human()
    );
}

const CONDITIONAL_OWNER: &str = r#"
[[owner]]
name = "conditional-process"
kind = "call"
within = ["src/**"]
match = "std::process::Command::new"
allow = ["src/bindings.rs"]
reason = "Only the reviewed binding owner may construct child processes."
"#;
