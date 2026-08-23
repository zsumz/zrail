//! Deep namespace controls keep local modules from borrowing external authority.

use zrail_core::{AnalysisQuality, Report};

use super::{assert_no_owner_findings, check, fixture, reset, write, write_executor, write_lock};

#[path = "namespace_deep_audit/editions.rs"]
mod editions;
#[path = "namespace_deep_audit/macro_opacity.rs"]
mod macro_opacity;
#[path = "namespace_deep_audit/opaque_members.rs"]
mod opaque_members;
#[path = "namespace_deep_audit/proc_macro_opacity.rs"]
mod proc_macro_opacity;
#[path = "namespace_deep_audit/qualifier_blocks.rs"]
mod qualifier_blocks;
#[path = "namespace_deep_audit/value_namespace.rs"]
mod value_namespace;
#[path = "namespace_deep_audit/visibility.rs"]
mod visibility;

#[test]
fn parent_std_alias_does_not_hide_external_std_in_an_inline_child() {
    let root = fixture("namespace-parent-std", "");
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\nmod local_std {}\nuse crate::local_std as std;\nmod child { pub fn hidden() { let _ = std::process::Command::new(\"sh\"); } }\n",
    );
    write_executor(&root);
    write_lock(&root);

    assert_exact_owned_call(&check(&root), "src/lib.rs");
    reset(&root);
}

#[test]
fn parent_std_alias_does_not_leak_into_a_child_declared_by_an_include() {
    let root = fixture("namespace-included-child-std", "");
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\nmod local_std {}\nuse crate::local_std as std;\ninclude!(\"child.rs\");\n",
    );
    write(
        &root,
        "src/child.rs",
        "mod child { pub fn hidden() { let _ = std::process::Command::new(\"sh\"); } }\n",
    );
    write_executor(&root);
    write_lock(&root);

    assert_exact_owned_call(&check(&root), "src/child.rs");
    reset(&root);
}

#[test]
fn nested_module_globs_reach_a_fixed_point() {
    let root = fixture("namespace-glob-fixed-point", "");
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\nmod prelude;\nuse prelude::*;\nuse bridge::*;\npub fn hidden() { let _ = Command::new(\"sh\"); }\n",
    );
    write(
        &root,
        "src/prelude.rs",
        "//! Prelude.\npub mod bridge { pub use std::process::Command; }\n",
    );
    write_executor(&root);
    write_lock(&root);

    let report = check(&root);
    let finding = owned_call(&report, "process-spawn", "src/lib.rs");
    assert_ne!(
        finding.analysis,
        AnalysisQuality::Unresolved,
        "{}",
        report.human()
    );
    reset(&root);
}

#[test]
fn local_reexport_keeps_its_canonical_module_prefix() {
    let root = fixture("namespace-local-canonical", LOCAL_CONTRACT);
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\nmod owner;\nmod bridge { pub struct Spawn; impl Spawn { pub fn new(_: &str) -> Self { Self } } pub use self::Spawn as Process; }\npub fn hidden() { let _ = bridge::Process::new(\"local\"); }\n",
    );
    write(
        &root,
        "src/owner.rs",
        "//! Local owner.\npub fn allowed() { let _ = crate::bridge::Spawn::new(\"local\"); }\n",
    );
    write_executor(&root);
    write_lock(&root);

    let report = check(&root);
    assert_exact(&report, "OWN-003", "local-construction", "src/lib.rs");
    assert_exact(&report, "CAP-001", "local-symbols", "src/lib.rs");
    assert!(
        !report.findings.iter().any(|finding| {
            matches!(finding.id.as_str(), "OWN-004" | "OWN-005")
                && finding.rule == "local-construction"
        }),
        "{}",
        report.human()
    );
    reset(&root);
}

#[test]
fn exact_local_module_function_remains_a_negative_control() {
    let root = fixture("namespace-local-function", "");
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\nmod bridge { pub fn spawn() {} }\npub fn local() { bridge::spawn(); }\n",
    );
    write_executor(&root);
    write_lock(&root);

    let report = check(&root);
    assert_no_owner_findings(&report, "process-spawn");
    assert_no_include_finding(&report);
    reset(&root);
}

#[test]
fn macro_introduced_unknown_member_fails_closed() {
    let root = fixture("namespace-macro-member", MACRO_MEMBER_CONTRACT);
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\nmacro_rules! members { () => { pub struct Spawn; impl Spawn { pub fn new(_: &str) -> Self { Self } } } }\nmod bridge { members!(); }\npub fn allowed() { let _ = bridge::Spawn::new(\"local\"); }\n",
    );
    write_executor(&root);
    write_lock(&root);

    let report = check(&root);
    assert!(
        report.findings.iter().any(|finding| {
            finding.id == "RUST-INCLUDE-002"
                || (finding.rule == "macro-member"
                    && finding.id.starts_with("OWN-")
                    && finding.analysis != AnalysisQuality::Exact)
        }),
        "{}",
        report.human()
    );
    reset(&root);
}

#[test]
fn allowed_block_alias_remains_exact() {
    assert_allowed_alias(
        "block",
        "pub fn allowed() { use std::process::Command as Spawn; let _ = Spawn::new(\"true\"); }",
    );
}

#[test]
fn allowed_type_alias_remains_exact() {
    assert_allowed_alias(
        "type",
        "type Spawn = std::process::Command; pub fn allowed() { let _ = Spawn::new(\"true\"); }",
    );
}

fn assert_allowed_alias(name: &str, declaration: &str) {
    let root = fixture(&format!("namespace-allowed-{name}"), "");
    write(&root, "src/lib.rs", "//! Library.\nmod executor;\n");
    write(
        &root,
        "src/executor.rs",
        &format!("//! Authorized process owner.\n{declaration}\n"),
    );
    write_lock(&root);

    assert_no_owner_findings(&check(&root), "process-spawn");
    reset(&root);
}

fn assert_exact_owned_call(report: &Report, path: &str) {
    let finding = owned_call(report, "process-spawn", path);
    assert_eq!(
        finding.analysis,
        AnalysisQuality::Exact,
        "{}",
        report.human()
    );
}

fn owned_call<'a>(report: &'a Report, rule: &str, path: &str) -> &'a zrail_core::Finding {
    report
        .findings
        .iter()
        .find(|finding| {
            finding.id == "OWN-003" && finding.rule == rule && finding.path.as_deref() == Some(path)
        })
        .unwrap_or_else(|| panic!("{}", report.human()))
}

fn assert_exact(report: &Report, id: &str, rule: &str, path: &str) {
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

fn assert_no_include_finding(report: &Report) {
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.id == "RUST-INCLUDE-002"),
        "{}",
        report.human()
    );
}

const LOCAL_CONTRACT: &str = r#"
[[scope]]
name = "local-symbols"
include = ["src/lib.rs"]
reason = "Local aliases retain their canonical module prefix."
[scope.symbols]
deny = ["bridge::Spawn::new"]

[[owner]]
name = "local-construction"
kind = "call"
within = ["src/**"]
match = "bridge::Spawn::new"
allow = ["src/owner.rs"]
reason = "Only the local owner may construct this type."
"#;

const MACRO_MEMBER_CONTRACT: &str = r#"
[[owner]]
name = "macro-member"
kind = "call"
within = ["src/**"]
match = "bridge::Spawn::new"
allow = ["src/lib.rs"]
reason = "Macro-introduced members cannot receive exact authority."
"#;
