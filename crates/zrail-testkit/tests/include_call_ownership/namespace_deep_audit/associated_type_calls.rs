//! Associated-type call projections cannot borrow exact direct-call authority.

use zrail_core::{AnalysisQuality, Report};

use super::super::{check, fixture, reset, write, write_executor, write_lock};

#[test]
fn direct_associated_type_projection_fails_closed() {
    let root = fixture("associated-call-direct", "");
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\ntrait Provider { type Command; }\nstruct Runtime;\nimpl Provider for Runtime { type Command = std::process::Command; }\npub fn hidden() { let _ = <Runtime as Provider>::Command::new(\"sh\"); }\n",
    );
    write_executor(&root);
    write_lock(&root);

    let report = check(&root);
    assert_unresolved_call(&report, "src/lib.rs", "<Runtime as Provider>::Command::new");
    assert_owner_is_fresh(&report);
    reset(&root);
}

#[test]
fn associated_type_projection_inside_allowed_owner_is_not_exact() {
    let root = fixture("associated-call-owner", "");
    write(&root, "src/lib.rs", "//! Library.\nmod executor;\n");
    write(
        &root,
        "src/executor.rs",
        "//! Authorized process owner.\ntrait Provider { type Command; }\nstruct Runtime;\nimpl Provider for Runtime { type Command = std::process::Command; }\npub fn allowed() { let _ = <Runtime as Provider>::Command::new(\"true\"); }\n",
    );
    write_lock(&root);

    let report = check(&root);
    assert_unresolved_call(
        &report,
        "src/executor.rs",
        "<Runtime as Provider>::Command::new",
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| { finding.id == "OWN-004" && finding.rule == "process-spawn" }),
        "{}",
        report.human()
    );
    reset(&root);
}

#[test]
fn associated_type_projection_through_imported_trait_fails_closed() {
    let root = fixture("associated-call-import", "");
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\nmod provider { pub trait Provider { type Command; } }\nuse provider::Provider as Contract;\nstruct Runtime;\nimpl Contract for Runtime { type Command = std::process::Command; }\npub fn hidden() { let _ = <Runtime as Contract>::Command::new(\"sh\"); }\n",
    );
    write_executor(&root);
    write_lock(&root);

    let report = check(&root);
    assert_unresolved_call(&report, "src/lib.rs", "<Runtime as Contract>::Command::new");
    assert_owner_is_fresh(&report);
    reset(&root);
}

#[test]
fn direct_trait_function_remains_exact() {
    let root = fixture("associated-call-trait", TRAIT_CALL_CONTRACT);
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\npub fn allowed() { let _ = <String as std::convert::From<&str>>::from(\"value\"); }\n",
    );
    write_executor(&root);
    write_lock(&root);

    let report = check(&root);
    assert!(
        report.findings.iter().any(|finding| {
            finding.id == "CAP-001"
                && finding.rule == "trait-call"
                && finding.path.as_deref() == Some("src/lib.rs")
                && finding.analysis == AnalysisQuality::Exact
        }),
        "{}",
        report.human()
    );
    assert_no_call_resolution_finding(&report);
    reset(&root);
}

fn assert_unresolved_call(report: &Report, path: &str, written: &str) {
    assert!(
        report.findings.iter().any(|finding| {
            finding.id == "RUST-CALL-001"
                && finding.rule == "rust.source.call-resolution"
                && finding.path.as_deref() == Some(path)
                && finding.analysis == AnalysisQuality::Unresolved
                && finding.message.contains(written)
        }),
        "{}",
        report.human()
    );
}

fn assert_no_call_resolution_finding(report: &Report) {
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.id == "RUST-CALL-001"),
        "{}",
        report.human()
    );
}

fn assert_owner_is_fresh(report: &Report) {
    assert!(
        !report.findings.iter().any(|finding| {
            matches!(finding.id.as_str(), "OWN-004" | "OWN-005") && finding.rule == "process-spawn"
        }),
        "{}",
        report.human()
    );
}

const TRAIT_CALL_CONTRACT: &str = r#"
[[scope]]
name = "trait-call"
include = ["src/lib.rs"]
reason = "The direct named trait call retains its exact identity."
[scope.symbols]
deny = ["std::convert::From::from"]
"#;
