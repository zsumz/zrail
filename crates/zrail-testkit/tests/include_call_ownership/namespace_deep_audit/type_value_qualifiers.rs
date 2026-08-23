//! Type qualifiers ignore same-name values re-exported beside their module.

use zrail_core::AnalysisQuality;

use super::super::{check, fixture, reset, write, write_executor, write_lock};

#[test]
fn module_qualifier_wins_over_a_same_name_function_reexport() {
    let root = fixture("namespace-module-function-qualifier", CONTRACT);
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\nmod rules;\n",
    );
    std::fs::create_dir_all(root.join("src/rules")).expect("create nested rules fixture");
    write(
        &root,
        "src/rules/mod.rs",
        "//! Rules.\nmod evaluate;\nmod capability;\npub(crate) use evaluate::{RuleContext, evaluate};\n",
    );
    write(
        &root,
        "src/rules/evaluate.rs",
        "//! Evaluation.\npub(crate) struct RuleContext<'a> { _marker: &'a () }\npub(crate) fn evaluate(_: &RuleContext<'_>) {}\n",
    );
    write(
        &root,
        "src/rules/capability.rs",
        "//! Capability.\nuse super::RuleContext;\npub(super) fn evaluate(_: &RuleContext<'_>) {}\n",
    );
    write_executor(&root);
    write_lock(&root);

    let report = check(&root);
    assert!(
        report.findings.iter().any(|finding| {
            finding.id == "CAP-001"
                && finding.rule == "rule-context"
                && finding.path.as_deref() == Some("src/rules/capability.rs")
                && finding.analysis == AnalysisQuality::Exact
        }),
        "{}",
        report.human()
    );
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
fn local_function_wins_over_an_imported_same_name_module() {
    let root = fixture(
        "namespace-imported-module-local-function",
        LOCAL_CALL_CONTRACT,
    );
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\nmod manifest;\nmod consumer;\n",
    );
    write(
        &root,
        "src/manifest.rs",
        "//! Manifest module.\npub struct Entry;\n",
    );
    write(
        &root,
        "src/consumer.rs",
        "//! Consumer.\nuse super::manifest;\nfn manifest() {}\npub fn hidden() { manifest(); }\n",
    );
    write_executor(&root);
    write_lock(&root);

    let report = check(&root);
    assert!(
        report.findings.iter().any(|finding| {
            finding.id == "OWN-003"
                && finding.rule == "local-manifest"
                && finding.path.as_deref() == Some("src/consumer.rs")
                && finding.analysis == AnalysisQuality::Exact
        }),
        "{}",
        report.human()
    );
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
fn opaque_alias_name_is_exact_in_a_plain_type_position() {
    let root = fixture("namespace-opaque-local-type", OPAQUE_TYPE_CONTRACT);
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\ntype Local = (u8, u8);\npub fn accept(_: Local) {}\n",
    );
    write_executor(&root);
    write_lock(&root);

    let report = check(&root);
    assert!(
        report.findings.iter().any(|finding| {
            finding.id == "CAP-001"
                && finding.rule == "opaque-local-type"
                && finding.path.as_deref() == Some("src/lib.rs")
                && finding.analysis == AnalysisQuality::Exact
        }),
        "{}",
        report.human()
    );
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

const CONTRACT: &str = r#"
[[scope]]
name = "rule-context"
include = ["src/rules/capability.rs"]
reason = "Type qualifiers resolve through the module namespace."
[scope.symbols]
deny = ["rules::evaluate::RuleContext"]
"#;

const LOCAL_CALL_CONTRACT: &str = r#"
[[owner]]
name = "local-manifest"
kind = "call"
within = ["src/**"]
match = "consumer::manifest"
allow = ["src/executor.rs"]
reason = "The local function remains in the value namespace."
"#;

const OPAQUE_TYPE_CONTRACT: &str = r#"
[[scope]]
name = "opaque-local-type"
include = ["src/lib.rs"]
reason = "The alias name is exact even when its right-hand side is opaque."
[scope.symbols]
deny = ["Local"]
"#;
