//! Binding-clean review removes only the exact macro boundary it proves safe.

use std::{fs, path::PathBuf};

use zrail_core::{AnalysisQuality, Report, ReportStatus};

use super::{MANIFEST, check, repository, reset};

#[test]
fn exact_source_bound_serde_review_keeps_the_namespace_complete() {
    let root = serde_fixture("binding-clean-serde", true);

    let report = check(&root);

    assert_eq!(report.status, ReportStatus::Pass, "{}", report.human());
    assert_no_macro_rejection(&report);
    assert_no_include_failure(&report);
    reset(&root);
}

#[test]
fn ordinary_source_bound_serde_review_keeps_the_namespace_opaque() {
    let root = serde_fixture("binding-opaque-serde", false);

    let report = check(&root);

    assert_no_macro_rejection(&report);
    assert_include_failure(&report);
    reset(&root);
}

#[test]
fn wrong_source_attribute_cannot_borrow_binding_clean_authority() {
    assert_wrong_source_fails_closed(
        "binding-spoofed-attribute",
        "emit_import::inject_spawn",
        "//! Spoofed attribute.\n#[emit_import::inject_spawn]\npub struct Marker;\npub fn run() { let _ = Spawn::new(\"sh\"); }\n",
    );
}

#[test]
fn wrong_source_derive_cannot_borrow_binding_clean_authority() {
    assert_wrong_source_fails_closed(
        "binding-spoofed-derive",
        "emit_import::InjectSpawn",
        "//! Spoofed derive.\n#[derive(emit_import::InjectSpawn)]\npub struct Marker;\npub fn run() { let _ = Spawn::new(\"sh\"); }\n",
    );
}

fn serde_fixture(name: &str, binding_clean: bool) -> PathBuf {
    let bindings = if binding_clean {
        "bindings = \"none\"\n"
    } else {
        ""
    };
    let allowances = serde_allowances(bindings);
    let root = repository(name, SERDE_SOURCE, &allowances);
    fs::write(
        root.join("Cargo.toml"),
        format!(
            "{MANIFEST}\n[dependencies]\nserde = {{ package = \"serde\", version = \"=1.0.229\", features = [\"derive\"] }}\n"
        ),
    )
    .expect("write serde dependency");
    if binding_clean {
        zrail_rust::build_lock(&root, "zrail.toml".as_ref())
            .expect("build complete serde fixture lock")
            .write(&root.join("zrail.lock"))
            .expect("write complete serde fixture lock");
    }
    root
}

fn serde_allowances(bindings: &str) -> String {
    format!(
        r#"
[[source.rust.macros.allow]]
name = "serde"
{bindings}reason = "Reviewed serde helper output preserves the ordinary namespace exactly."
[source.rust.macros.allow.source]
kind = "registry"
requirement = "=1.0.229"

[[source.rust.macros.allow]]
name = "serde::Deserialize"
{bindings}reason = "Reviewed serde derive output preserves the ordinary namespace exactly."
[source.rust.macros.allow.source]
kind = "registry"
requirement = "=1.0.229"
"#
    )
}

fn assert_wrong_source_fails_closed(name: &str, macro_name: &str, source: &str) {
    let allowance = wrong_source_allowance(macro_name);
    let root = proc_macro_fixture(name, source, &allowance);

    let report = check(&root);

    let mismatches = report
        .findings
        .iter()
        .filter(|finding| finding.id == "RUST-MACRO-006" && finding.message.contains(macro_name))
        .count();
    assert_eq!(mismatches, 1, "{}", report.human());
    assert_include_failure(&report);
    reset(&root);
}

fn wrong_source_allowance(name: &str) -> String {
    format!(
        r#"
[[source.rust.macros.allow]]
name = "{name}"
bindings = "none"
reason = "Only a reviewed registry implementation may preserve the ordinary namespace exactly."
[source.rust.macros.allow.source]
kind = "registry"
requirement = "=1.0.0"
"#
    )
}

fn proc_macro_fixture(name: &str, source: &str, allowance: &str) -> PathBuf {
    let root = repository(name, source, allowance);
    fs::create_dir_all(root.join("emit_import/src")).expect("create proc-macro fixture");
    fs::write(root.join("Cargo.toml"), PROC_MACRO_WORKSPACE).expect("write workspace manifest");
    fs::write(root.join("emit_import/Cargo.toml"), PROC_MACRO_MANIFEST)
        .expect("write proc-macro manifest");
    fs::write(root.join("emit_import/src/lib.rs"), PROC_MACRO_SOURCE)
        .expect("write proc-macro source");
    root
}

fn assert_no_macro_rejection(report: &Report) {
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| { matches!(finding.id.as_str(), "RUST-MACRO-001" | "RUST-MACRO-006") }),
        "{}",
        report.human()
    );
}

fn assert_no_include_failure(report: &Report) {
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.id == "RUST-INCLUDE-002"),
        "{}",
        report.human()
    );
}

fn assert_include_failure(report: &Report) {
    assert!(
        report.findings.iter().any(|finding| {
            finding.id == "RUST-INCLUDE-002"
                && finding.path.as_deref() == Some("src/lib.rs")
                && finding.analysis == AnalysisQuality::Unresolved
        }),
        "{}",
        report.human()
    );
}

const SERDE_SOURCE: &str = r"//! Serde binding fixture.
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Model {
    value: Option<String>,
}

mod consumer {
    use super::Model;

    pub fn clean(_: Model) -> Vec<String> {
        Vec::new()
    }
}
";

const PROC_MACRO_WORKSPACE: &str = r#"[workspace]
members = [".", "emit_import"]
resolver = "3"

[package]
name = "fixture"
version = "0.0.0"
edition = "2024"

[dependencies]
emit_import = { path = "emit_import" }
"#;

const PROC_MACRO_MANIFEST: &str = r#"[package]
name = "emit_import"
version = "0.0.0"
edition = "2024"

[lib]
proc-macro = true
"#;

const PROC_MACRO_SOURCE: &str = r#"//! Test proc macros.

extern crate proc_macro;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn inject_spawn(_: TokenStream, item: TokenStream) -> TokenStream {
    let mut output: TokenStream = "use std::process::Command as Spawn;".parse().unwrap();
    output.extend(item);
    output
}

#[proc_macro_derive(InjectSpawn)]
pub fn derive_spawn(_: TokenStream) -> TokenStream {
    "use std::process::Command as Spawn;".parse().unwrap()
}
"#;
