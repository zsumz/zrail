//! Opaque macro output cannot bypass exact source-operation owners.

use std::{fs, path::PathBuf};

use zrail_core::{AnalysisQuality, Report};
use zrail_rust::{check_repository, governed_surface_report};

#[test]
fn every_operation_owner_fails_closed_for_every_macro_form() {
    let root = repository();
    let report = check(&root);

    for rule in OPERATION_OWNERS {
        for (id, path) in [("OWN-006", "src/owner.rs"), ("OWN-003", "src/outside.rs")] {
            assert!(
                report.findings.iter().any(|finding| {
                    finding.rule == rule
                        && finding.id == id
                        && finding.path.as_deref() == Some(path)
                        && finding.analysis == AnalysisQuality::Unresolved
                }),
                "missing {rule} {id} at {path}: {}",
                report.human(),
            );
        }
    }
    for path in ["src/owner.rs", "src/outside.rs"] {
        let findings = report
            .findings
            .iter()
            .filter(|finding| {
                finding.rule == "field-read"
                    && finding.path.as_deref() == Some(path)
                    && finding.message.contains("macro expansion")
            })
            .collect::<Vec<_>>();
        assert!(
            findings
                .iter()
                .filter(|finding| finding.message.contains("emit"))
                .count()
                >= 3,
            "item, statement, and expression macros were not all closed: {}",
            report.human(),
        );
        for name in ["OpaqueDerive", "opaque_attr"] {
            assert!(
                findings
                    .iter()
                    .any(|finding| finding.message.contains(name)),
                "missing {name} boundary at {path}: {}",
                report.human(),
            );
        }
    }
    assert!(
        !report.findings.iter().any(|finding| {
            finding.id.starts_with("OWN-") && finding.path.as_deref() == Some("src/attested.rs")
        }),
        "exact source-operation-free attestation did not close the boundary: {}",
        report.human(),
    );
    let coverage =
        governed_surface_report(&root, "zrail.toml".as_ref()).expect("build macro coverage");
    let field_owner = coverage
        .owners
        .iter()
        .find(|owner| owner.name == "field-read")
        .expect("field owner coverage");
    assert!(field_owner.occurrences.iter().any(|occurrence| {
        occurrence.operation == "opaque-macro-source-operation"
            && occurrence.path == "src/outside.rs"
            && occurrence.quality == AnalysisQuality::Unresolved
    }));
    assert!(
        !field_owner
            .occurrences
            .iter()
            .any(|occurrence| occurrence.path == "src/attested.rs")
    );
    reset(&root);
}

fn repository() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "zrail-macro-operation-owner-{}-{:?}",
        std::process::id(),
        std::thread::current().id(),
    ));
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create fixture source");
    write(&root, "Cargo.toml", MANIFEST);
    write(&root, "zrail.toml", CONTRACT);
    write(&root, "src/lib.rs", LIBRARY);
    write(&root, "src/owner.rs", OPAQUE_FORMS);
    write(&root, "src/outside.rs", OPAQUE_FORMS);
    write(&root, "src/attested.rs", ATTESTED);
    root
}

fn check(root: &std::path::Path) -> Report {
    check_repository(root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check macro operation fixture")
        .report
}

fn write(root: &std::path::Path, path: &str, contents: &str) {
    fs::write(root.join(path), contents).expect("write fixture");
}

fn reset(root: &PathBuf) {
    if root.exists() {
        fs::remove_dir_all(root).expect("reset fixture");
    }
}

const OPERATION_OWNERS: [&str; 7] = [
    "construction",
    "method-name",
    "field-read",
    "field-write",
    "field-borrow",
    "field-mutation",
    "field-authority",
];

const MANIFEST: &str = r#"[package]
name = "macro-operation-fixture"
version = "0.0.0"
edition = "2024"
"#;

const LIBRARY: &str = r#"//! Macro operation fixture.
macro_rules! emit { () => {} }
macro_rules! clean { () => {} }
#[path = "owner.rs"] mod owner;
#[path = "outside.rs"] mod outside;
#[path = "attested.rs"] mod attested;
pub struct State { pub epoch: usize }
"#;

const OPAQUE_FORMS: &str = r"//! Opaque macro forms.
#[opaque_attr]
#[derive(OpaqueDerive)]
struct Model;
emit!();
fn run() {
    emit!();
    let _ = emit!();
}
";

const ATTESTED: &str = r"//! Attested macro form.
pub fn run() { clean!(); }
";

const CONTRACT: &str = include_str!("macro_operation_ownership.toml");
