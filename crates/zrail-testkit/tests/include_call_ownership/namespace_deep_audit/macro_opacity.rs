//! Macro-generated imports cannot receive exact authority across statement or include scopes.

use zrail_core::{AnalysisQuality, Report};

use super::super::{check, fixture, reset, write, write_executor, write_lock};

#[test]
fn statement_macro_generated_import_fails_closed() {
    let root = fixture("namespace-statement-macro-import", MACRO_IMPORT_CONTRACT);
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\nmacro_rules! bind_spawn { () => { use std::process::Command as Spawn; }; }\npub fn allowed() { bind_spawn!(); let _ = Spawn::new(\"sh\"); }\n",
    );
    write_executor(&root);
    write_lock(&root);

    assert_fail_closed(&check(&root), "statement-macro-import");
    reset(&root);
}

#[test]
fn included_item_macro_generated_import_fails_closed() {
    let root = fixture(
        "namespace-included-item-macro-import",
        MACRO_IMPORT_CONTRACT,
    );
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\nmacro_rules! bind_spawn { () => { use std::process::Command as Spawn; }; }\ninclude!(\"imports.rs\");\npub fn allowed() { let _ = Spawn::new(\"sh\"); }\n",
    );
    write(&root, "src/imports.rs", "bind_spawn!();\n");
    write_executor(&root);
    write_lock(&root);

    assert_fail_closed(&check(&root), "included-item-macro-import");
    reset(&root);
}

#[test]
fn opaque_explicit_import_takes_precedence_over_a_known_benign_glob() {
    let root = fixture("namespace-opaque-over-benign-glob", PRECEDENCE_CONTRACT);
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\npub struct Benign;\nimpl Benign { pub fn new(_: &str) -> Self { Self } }\nmod benign { pub use crate::Benign as Spawn; }\nuse benign::*;\nmacro_rules! bind_spawn { () => { use std::process::Command as Spawn; }; }\nbind_spawn!();\nuse crate::Benign as Known;\npub fn allowed() { let _ = Spawn::new(\"sh\"); }\npub fn control() { let _ = Known::new(\"local\"); }\n",
    );
    write_executor(&root);
    write_lock(&root);

    let report = check(&root);
    assert_fail_closed(&report, "opaque-over-benign-glob");
    assert!(
        report.findings.iter().any(|finding| {
            finding.id == "OWN-003"
                && finding.rule == "known-explicit-control"
                && finding.path.as_deref() == Some("src/lib.rs")
                && finding.analysis == AnalysisQuality::Exact
        }),
        "{}",
        report.human()
    );
    reset(&root);
}

fn assert_fail_closed(report: &Report, rule: &str) {
    assert!(
        report.findings.iter().any(|finding| {
            finding.id == "RUST-INCLUDE-002"
                || (finding.id.starts_with("OWN-")
                    && finding.rule == rule
                    && finding.analysis != AnalysisQuality::Exact)
        }),
        "{}",
        report.human()
    );
}

const MACRO_IMPORT_CONTRACT: &str = r#"
[[owner]]
name = "statement-macro-import"
kind = "call"
within = ["src/**"]
match = "Spawn::new"
allow = ["src/lib.rs"]
reason = "Statement macro imports cannot receive exact direct-call authority."

[[owner]]
name = "included-item-macro-import"
kind = "call"
within = ["src/**"]
match = "Spawn::new"
allow = ["src/lib.rs"]
reason = "Included item macro imports cannot receive exact direct-call authority."
"#;

const PRECEDENCE_CONTRACT: &str = r#"
[[owner]]
name = "opaque-over-benign-glob"
kind = "call"
within = ["src/**"]
match = "Spawn::new"
allow = ["src/lib.rs"]
reason = "An opaque explicit binding takes precedence over a known glob."

[[owner]]
name = "known-explicit-control"
kind = "call"
within = ["src/**"]
match = "Benign::new"
allow = ["src/executor.rs"]
reason = "Known explicit aliases remain exact despite unrelated namespace opacity."
"#;
