//! Absolute and plain imports retain their Rust-edition namespace semantics.

use zrail_core::{AnalysisQuality, Report};

use super::super::{
    assert_no_owner_findings, check, fixture, reset, write, write_executor, write_lock,
};

#[test]
fn included_absolute_std_import_ignores_a_local_std_module_in_2024() {
    let root = edition_fixture("absolute-use", "2024", "");
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\nmod std {}\ninclude!(\"imports.rs\");\npub fn hidden() { let _ = Spawn::new(\"sh\"); }\n",
    );
    write(
        &root,
        "src/imports.rs",
        "use ::std::process::Command as Spawn;\n",
    );
    finish_exact_process(&root, "src/lib.rs");
}

#[test]
fn absolute_type_alias_resolves_external_std_in_2015() {
    absolute_type_alias("2015");
}

#[test]
fn absolute_type_alias_resolves_external_std_in_2024() {
    absolute_type_alias("2024");
}

#[test]
fn plain_item_use_selects_the_crate_root_in_2015() {
    let root = item_use_fixture("2015", "");
    finish_exact_process(&root, "src/lib.rs");
}

#[test]
fn plain_item_use_selects_the_local_module_in_2024() {
    let root = item_use_fixture("2024", BENIGN_CONTRACT);
    write_executor(&root);
    write_lock(&root);

    let report = check(&root);
    assert_no_owner_findings(&report, "process-spawn");
    assert_exact(&report, "CAP-001", "benign-symbol", "src/lib.rs");
    reset(&root);
}

fn absolute_type_alias(edition: &str) {
    let root = edition_fixture("absolute-type", edition, "");
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\ntype Spawn = ::std::process::Command;\npub fn hidden() { let _ = Spawn::new(\"sh\"); }\n",
    );
    finish_exact_process(&root, "src/lib.rs");
}

fn item_use_fixture(edition: &str, contract: &str) -> std::path::PathBuf {
    let root = edition_fixture("plain-item-use", edition, contract);
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\nmod process { pub use std::process::Command; }\npub struct Benign;\nimpl Benign { fn new(_: &str) -> Self { Self } }\nmod outer { mod process { pub use super::super::Benign as Command; } use process::Command as Spawn; fn hidden() { let _ = Spawn::new(\"sh\"); } }\n",
    );
    root
}

fn edition_fixture(name: &str, edition: &str, contract: &str) -> std::path::PathBuf {
    let root = fixture(&format!("namespace-{name}-{edition}"), contract);
    write(
        &root,
        "Cargo.toml",
        &format!("[package]\nname = \"fixture\"\nversion = \"0.0.0\"\nedition = \"{edition}\"\n"),
    );
    root
}

fn finish_exact_process(root: &std::path::Path, path: &str) {
    write_executor(root);
    write_lock(root);
    let report = check(root);
    assert_exact(&report, "OWN-003", "process-spawn", path);
    reset(root);
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

const BENIGN_CONTRACT: &str = r#"
[[scope]]
name = "benign-symbol"
include = ["src/lib.rs"]
reason = "Edition 2024 resolves the plain import to the local benign module."
[scope.symbols]
deny = ["Benign::new"]
"#;
