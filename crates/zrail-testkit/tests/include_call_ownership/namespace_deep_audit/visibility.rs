//! Module binding visibility follows Rust's descendant and restricted scopes.

use zrail_core::{AnalysisQuality, Report};

use super::super::{
    assert_no_owner_findings, check, fixture, reset, write, write_executor, write_lock,
};

#[test]
fn value_and_type_namespaces_coexist_without_false_authority() {
    let root = repository(
        "namespace-coexistence",
        "//! Library.\nmod executor;\nmod foo {}\nfn foo() {}\nmod bridge { pub fn spawn() {} }\npub fn clean() { foo(); bridge::spawn(); }\n",
    );
    let report = check(&root);
    assert_no_owner_findings(&report, "process-spawn");
    assert_no_include(&report);
    reset(&root);
}

#[test]
fn private_binding_is_exact_for_a_descendant() {
    assert_exact_source(
        "visibility-private-descendant",
        "//! Library.\nmod executor;\nmod outer { use std::process::Command as Spawn; mod inner { pub fn hidden() { let _ = super::Spawn::new(\"sh\"); } } }\n",
    );
}

#[test]
fn private_child_binding_is_unresolved_from_its_parent() {
    assert_unresolved_source(
        "visibility-private-parent",
        "//! Library.\nmod executor;\nmod bridge { use std::process::Command as Spawn; }\npub fn hidden() { let _ = bridge::Spawn::new(\"sh\"); }\n",
    );
}

#[test]
fn private_binding_is_unresolved_from_a_sibling() {
    assert_unresolved_source(
        "visibility-private-sibling",
        "//! Library.\nmod executor;\nmod left { use std::process::Command as Spawn; }\nmod right { pub fn hidden() { let _ = super::left::Spawn::new(\"sh\"); } }\n",
    );
}

#[test]
fn pub_super_binding_is_exact_for_its_parent() {
    assert_exact_source(
        "visibility-pub-super",
        "//! Library.\nmod executor;\nmod bridge { pub(super) use std::process::Command as Spawn; }\npub fn hidden() { let _ = bridge::Spawn::new(\"sh\"); }\n",
    );
}

#[test]
fn pub_crate_binding_is_exact_for_a_sibling() {
    assert_exact_source(
        "visibility-pub-crate",
        "//! Library.\nmod executor;\nmod bridge { pub(crate) use std::process::Command as Spawn; }\nmod sibling { pub fn hidden() { let _ = crate::bridge::Spawn::new(\"sh\"); } }\n",
    );
}

#[test]
fn pub_in_binding_is_exact_inside_its_restricted_module() {
    assert_exact_source(
        "visibility-pub-in",
        "//! Library.\nmod executor;\nmod outer { mod bridge { pub(in crate::outer) use std::process::Command as Spawn; } mod sibling { pub fn hidden() { let _ = super::bridge::Spawn::new(\"sh\"); } } }\n",
    );
}

#[test]
fn edition_2015_bare_pub_in_respects_crate_root_visibility() {
    let inside = edition_2015_repository(
        "visibility-pub-in-2015-inside",
        "//! Library.\nmod executor;\nmod outer { mod inner { pub(in outer) use std::process::Command as Spawn; } mod sibling { pub fn hidden() { let _ = super::inner::Spawn::new(\"sh\"); } } }\n",
    );
    assert_exact(&check(&inside), "src/lib.rs");
    reset(&inside);

    let outside = edition_2015_repository(
        "visibility-pub-in-2015-outside",
        "//! Library.\nmod executor;\nmod outer { pub mod inner { pub(in outer) use std::process::Command as Spawn; } }\nmod outside { pub fn hidden() { let _ = ::outer::inner::Spawn::new(\"sh\"); } }\n",
    );
    let report = check(&outside);
    assert!(
        report.findings.iter().any(|finding| {
            finding.id == "RUST-INCLUDE-002"
                || (finding.id == "OWN-003"
                    && finding.rule == "process-spawn"
                    && finding.path.as_deref() == Some("src/lib.rs")
                    && finding.analysis != AnalysisQuality::Exact)
        }),
        "{}",
        report.human()
    );
    reset(&outside);
}

#[test]
fn include_splice_is_transparent_to_private_descendant_visibility() {
    let root = fixture("visibility-include-private", "");
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\nmod outer { include!(\"visible.rs\"); mod inner { pub fn hidden() { let _ = super::Spawn::new(\"sh\"); } } }\n",
    );
    write(
        &root,
        "src/visible.rs",
        "use std::process::Command as Spawn;\n",
    );
    write_executor(&root);
    write_lock(&root);

    assert_exact(&check(&root), "src/lib.rs");
    reset(&root);
}

fn assert_exact_source(name: &str, source: &str) {
    let root = repository(name, source);
    assert_exact(&check(&root), "src/lib.rs");
    reset(&root);
}

fn assert_unresolved_source(name: &str, source: &str) {
    let root = repository(name, source);
    let report = check(&root);
    assert!(
        report.findings.iter().any(|finding| {
            finding.id == "RUST-INCLUDE-002"
                || (finding.id == "OWN-003"
                    && finding.rule == "process-spawn"
                    && finding.path.as_deref() == Some("src/lib.rs")
                    && finding.analysis != AnalysisQuality::Exact)
        }),
        "{}",
        report.human()
    );
    reset(&root);
}

fn repository(name: &str, source: &str) -> std::path::PathBuf {
    let root = fixture(name, "");
    write(&root, "src/lib.rs", source);
    write_executor(&root);
    write_lock(&root);
    root
}

fn edition_2015_repository(name: &str, source: &str) -> std::path::PathBuf {
    let root = fixture(name, "");
    write(
        &root,
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\nedition = \"2015\"\n",
    );
    write(&root, "src/lib.rs", source);
    write_executor(&root);
    write_lock(&root);
    root
}

fn assert_exact(report: &Report, path: &str) {
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

fn assert_no_include(report: &Report) {
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.id == "RUST-INCLUDE-002"),
        "{}",
        report.human()
    );
}
