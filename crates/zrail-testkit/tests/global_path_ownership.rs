//! Leading global roots keep edition and Cargo-alias authority through operations.

use std::{fs, path::PathBuf};

use zrail_core::{AnalysisQuality, Report};
use zrail_rust::check_repository;

#[test]
fn renamed_global_dependency_does_not_collapse_into_same_named_local_module() {
    let root = repository();
    let report = check(&root);

    assert_own003(&report, "external-construction", AnalysisQuality::Exact);
    assert_own003(&report, "local-construction", AnalysisQuality::Exact);
    assert_own003(&report, "external-field", AnalysisQuality::Unresolved);
    assert!(
        !report.findings.iter().any(|finding| {
            finding.id == "OWN-003"
                && finding.rule == "local-field"
                && finding.path.as_deref() == Some("src/trespasser.rs")
        }),
        "external update borrowed local fields: {}",
        report.human(),
    );
    fs::remove_dir_all(root).expect("remove global path fixture");
}

fn assert_own003(report: &Report, rule: &str, quality: AnalysisQuality) {
    let matches = report
        .findings
        .iter()
        .filter(|finding| {
            finding.id == "OWN-003"
                && finding.rule == rule
                && finding.path.as_deref() == Some("src/trespasser.rs")
                && finding.analysis == quality
        })
        .count();
    assert_eq!(
        matches,
        1,
        "expected one {rule} finding with {quality:?}: {}",
        report.human()
    );
}

fn repository() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "zrail-global-path-{}-{:?}",
        std::process::id(),
        std::thread::current().id(),
    ));
    if root.exists() {
        fs::remove_dir_all(&root).expect("reset global path fixture");
    }
    fs::create_dir_all(root.join("src")).expect("create source");
    for (path, contents) in FILES {
        fs::write(root.join(path), contents).expect("write global path fixture");
    }
    root
}

fn check(root: &std::path::Path) -> Report {
    check_repository(root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check global path fixture")
        .report
}

const FILES: &[(&str, &str)] = &[
    (
        "Cargo.toml",
        r#"[package]
name = "global-path-fixture"
version = "0.0.0"
edition = "2024"

[dependencies]
wire = { package = "wire-model", version = "1" }
"#,
    ),
    ("zrail.toml", CONTRACT),
    (
        "src/lib.rs",
        "//! Global path fixture.\nmod owner;\nmod trespasser;\nmod wire;\n",
    ),
    (
        "src/wire.rs",
        "//! Local collision.\npub struct Ticket { pub id: u64, pub local_secret: u64 }\n",
    ),
    (
        "src/owner.rs",
        r"//! Declared owners.
fn local() -> crate::wire::Ticket {
    let value = crate::wire::Ticket { id: 0, local_secret: 1 };
    let _ = value.local_secret;
    value
}
fn external(previous: ::wire::Ticket) -> ::wire::Ticket {
    ::wire::Ticket { id: 0, ..previous }
}
",
    ),
    (
        "src/trespasser.rs",
        r"//! Global and local trespassers.
fn local() -> crate::wire::Ticket {
    crate::wire::Ticket { id: 1, local_secret: 2 }
}
fn external(previous: ::wire::Ticket) -> ::wire::Ticket {
    ::wire::Ticket { id: 1, ..previous }
}
",
    ),
];

const CONTRACT: &str = r#"schema = 1
adapters = ["rust"]
[repository]
roots = ["."]
exclude = []
workspace_members = "exact"
nested_git = "deny"
submodules = "deny"
symlinks = "inside"
[dependencies]
mode = "observed"
unassigned_packages = "allow"
cycles = "deny"
[source.rust]
module_docs = "required"
facades = "allow"
tests = "allow"
[source.rust.macros]
mode = "allow"
[source.rust.hygiene]
unsafe = "deny"
lint_suppressions = "allow"

[[owner]]
name = "external-construction"
kind = "type-construction"
within = ["src/**"]
match = "wire_model::Ticket"
allow = ["src/owner.rs"]
reason = "External ticket construction stays centralized."

[[owner]]
name = "local-construction"
kind = "type-construction"
within = ["src/**"]
match = "crate::wire::Ticket"
allow = ["src/owner.rs"]
reason = "Local ticket construction stays centralized."

[[owner]]
name = "external-field"
kind = "field-read"
within = ["src/**"]
match = "wire_model::Ticket::external_secret"
allow = ["src/owner.rs"]
reason = "External ticket reads stay centralized."

[[owner]]
name = "local-field"
kind = "field-read"
within = ["src/**"]
match = "crate::wire::Ticket::local_secret"
allow = ["src/owner.rs"]
reason = "Local ticket reads stay centralized."
"#;
