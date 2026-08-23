//! Local module namespaces cannot hide reexported policy identities.

use zrail_core::{AnalysisQuality, Report};

use super::{
    PRODUCTION_OWNER, assert_no_owner_findings, assert_owned_call, check, fixture, reset, write,
    write_executor, write_lock,
};

#[test]
fn inline_module_reexport_cannot_bypass_call_ownership() {
    let root = fixture("module-inline-reexport", "");
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\nmod bridge { pub(crate) use std::process::Command as Spawn; }\npub fn hidden() { let _ = bridge::Spawn::new(\"sh\"); }\n",
    );
    write_executor(&root);
    write_lock(&root);

    assert_exact_owned_call(&check(&root), "src/lib.rs");
    reset(&root);
}

#[test]
fn external_module_reexport_reaches_every_ordinary_path_consumer() {
    let root = fixture("module-external-reexport", CONSUMER_CONTRACT);
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\nmod bridge;\npub fn hidden() { let _ = crate::bridge::Spawn::new(\"sh\"); }\n",
    );
    write(
        &root,
        "src/bridge.rs",
        "//! Bridge.\npub(crate) use std::process::Command as Spawn;\n",
    );
    write_executor(&root);
    write_lock(&root);

    let report = check(&root);
    assert_exact_owned_call(&report, "src/lib.rs");
    assert_exact_finding(&report, "OWN-003", "process-capability", "src/lib.rs");
    assert_exact_finding(&report, "CAP-001", "process-symbols", "src/lib.rs");
    assert_exact_finding(&report, "EFFECT-001", "profile.restricted", "src/lib.rs");
    reset(&root);
}

#[test]
fn include_introduced_module_reexport_cannot_bypass_call_ownership() {
    let root = fixture("module-included-reexport", "");
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\ninclude!(\"bridge.rs\");\npub fn hidden() { let _ = crate::bridge::Spawn::new(\"sh\"); }\n",
    );
    write(
        &root,
        "src/bridge.rs",
        "pub(crate) mod bridge { pub(crate) use std::process::Command as Spawn; }\n",
    );
    write_executor(&root);
    write_lock(&root);

    assert_exact_owned_call(&check(&root), "src/lib.rs");
    reset(&root);
}

#[test]
fn alias_imported_from_a_module_continues_through_its_namespace() {
    let root = fixture("module-imported-alias", "");
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\nmod bridge { pub(crate) use std::process::Command as Spawn; }\nuse crate::bridge::Spawn as Process;\npub fn hidden() { let _ = Process::new(\"sh\"); }\n",
    );
    write_executor(&root);
    write_lock(&root);

    assert_exact_owned_call(&check(&root), "src/lib.rs");
    reset(&root);
}

#[test]
fn imported_module_alias_continues_through_its_namespace() {
    let root = fixture("module-imported-module", "");
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\nmod bridge { pub(crate) use std::process::Command as Spawn; }\nuse crate::bridge as process_bridge;\npub fn hidden() { let _ = process_bridge::Spawn::new(\"sh\"); }\n",
    );
    write_executor(&root);
    write_lock(&root);

    assert_exact_owned_call(&check(&root), "src/lib.rs");
    reset(&root);
}

#[test]
fn module_glob_continues_through_the_member_namespace() {
    let root = fixture("module-glob", "");
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\nmod bridge { pub(crate) use std::process::Command; }\nuse crate::bridge::*;\npub fn hidden() { let _ = Command::new(\"sh\"); }\n",
    );
    write_executor(&root);
    write_lock(&root);

    assert_owned_call(&check(&root), "process-spawn", "src/lib.rs");
    reset(&root);
}

#[test]
fn chained_module_reexports_reach_the_original_identity() {
    let root = fixture("module-reexport-chain", "");
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\nmod origin { pub(crate) use std::process::Command as Spawn; }\nmod bridge { pub(crate) use crate::origin::Spawn as Process; }\npub fn hidden() { let _ = bridge::Process::new(\"sh\"); }\n",
    );
    write_executor(&root);
    write_lock(&root);

    assert_exact_owned_call(&check(&root), "src/lib.rs");
    reset(&root);
}

#[test]
fn external_module_declared_by_an_include_uses_its_occurrence() {
    let root = fixture("module-external-from-include", "");
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\ninclude!(\"declaration.rs\");\npub fn hidden() { let _ = crate::bridge::Spawn::new(\"sh\"); }\n",
    );
    write(
        &root,
        "src/declaration.rs",
        "#[path = \"bridge_impl.rs\"] mod bridge;\n",
    );
    write(
        &root,
        "src/bridge_impl.rs",
        "//! Bridge.\npub(crate) use std::process::Command as Spawn;\n",
    );
    write_executor(&root);
    write_lock(&root);

    assert_exact_owned_call(&check(&root), "src/lib.rs");
    reset(&root);
}

#[test]
fn local_type_terminal_does_not_borrow_external_authority() {
    let root = fixture("module-local-type", "");
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\nmod bridge { pub struct Spawn; impl Spawn { pub fn new(_: &str) -> Self { Self } } }\npub fn local() { let _ = bridge::Spawn::new(\"local\"); }\n",
    );
    write_executor(&root);
    write_lock(&root);

    let report = check(&root);
    assert_no_owner_findings(&report, "process-spawn");
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.id == "RUST-INCLUDE-002"),
        "{}",
        report.human()
    );
    reset(&root);
}

#[test]
fn nested_super_module_reexport_cannot_bypass_call_ownership() {
    let root = fixture("module-nested-super", "");
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\nmod outer { mod bridge { pub(crate) use std::process::Command as Spawn; } mod inner { pub fn hidden() { let _ = super::bridge::Spawn::new(\"sh\"); } } }\n",
    );
    write_executor(&root);
    write_lock(&root);

    assert_exact_owned_call(&check(&root), "src/lib.rs");
    reset(&root);
}

#[test]
fn module_aliases_stay_separate_across_compilation_domains() {
    let root = fixture("module-compilation-domains", PRODUCTION_OWNER);
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\n#[cfg(not(test))] mod bridge { pub(crate) use crate::Benign as Spawn; }\n#[cfg(test)] mod bridge { pub(crate) use std::process::Command as Spawn; }\npub fn hidden() { let _ = crate::bridge::Spawn::new(\"sh\"); }\npub struct Benign;\nimpl Benign { pub fn new(_: &str) -> Self { Self } }\n",
    );
    write_executor(&root);
    write_lock(&root);

    let report = check(&root);
    assert_exact_owned_call(&report, "src/lib.rs");
    assert_no_owner_findings(&report, "production-process");
    reset(&root);
}

#[test]
fn ambiguous_module_namespace_fails_closed_inside_the_owner() {
    let root = fixture("module-ambiguous", "");
    write(&root, "src/lib.rs", "//! Library.\nmod executor;\n");
    write(
        &root,
        "src/executor.rs",
        "//! Authorized process owner.\n#[cfg(unix)] mod bridge { pub(crate) use std::process::Command as Spawn; }\n#[cfg(windows)] mod bridge { pub(crate) use std::process::Command as Spawn; }\npub fn allowed() { let _ = self::bridge::Spawn::new(\"true\"); }\n",
    );
    write_lock(&root);

    let report = check(&root);
    assert!(
        report.findings.iter().any(|finding| {
            finding.id == "OWN-005"
                && finding.rule == "process-spawn"
                && finding.path.as_deref() == Some("src/executor.rs")
                && finding.analysis != AnalysisQuality::Exact
        }),
        "{}",
        report.human()
    );
    reset(&root);
}

fn assert_exact_owned_call(report: &Report, path: &str) {
    assert_exact_finding(report, "OWN-003", "process-spawn", path);
}

fn assert_exact_finding(report: &Report, id: &str, rule: &str, path: &str) {
    assert!(
        report.findings.iter().any(|finding| {
            finding.id == id
                && finding.rule == rule
                && finding.path.as_deref() == Some(path)
                && finding.analysis == AnalysisQuality::Exact
        }),
        "{}",
        report.human()
    );
}

const CONSUMER_CONTRACT: &str = r#"
[profiles.restricted.effects]
deny = ["process"]

[[layer]]
name = "application"
packages = ["fixture"]
profiles = ["restricted"]
reason = "The fixture must expose process effects."

[[scope]]
name = "process-symbols"
include = ["src/lib.rs"]
reason = "The fixture forbids process symbols in the caller."
[scope.symbols]
deny = ["std::process"]

[[owner]]
name = "process-capability"
kind = "capability"
within = ["src/**"]
match = "std::process"
allow = ["src/executor.rs", "src/bridge.rs"]
reason = "Only the executor and bridge may name process capabilities."
"#;
