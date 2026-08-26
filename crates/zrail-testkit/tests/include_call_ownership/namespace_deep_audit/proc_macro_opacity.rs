//! Proc-macro-generated imports cannot silently receive exact call authority.

use zrail_core::{AnalysisQuality, Report};

use super::super::{check, fixture, reset, write, write_executor, write_lock};

#[test]
fn attribute_macro_generated_import_fails_closed() {
    let root = proc_macro_fixture("namespace-attribute-macro-import", ATTRIBUTE_CONTRACT);
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\nuse emit_import::inject_spawn;\n#[inject_spawn]\npub struct Marker;\npub fn allowed() { let _ = Spawn::new(\"sh\"); }\n",
    );
    write_executor(&root);
    write_lock(&root);

    assert_unresolved_direct_owner(&check(&root), "attribute-macro-import");
    reset(&root);
}

#[test]
fn derive_macro_generated_import_fails_closed() {
    let root = proc_macro_fixture("namespace-derive-macro-import", DERIVE_CONTRACT);
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\nuse emit_import::InjectSpawn;\n#[derive(InjectSpawn)]\npub struct Marker;\npub fn allowed() { let _ = Spawn::new(\"sh\"); }\n",
    );
    write_executor(&root);
    write_lock(&root);

    assert_unresolved_direct_owner(&check(&root), "derive-macro-import");
    reset(&root);
}

#[test]
fn attribute_replaced_benign_import_fails_closed() {
    let root = proc_macro_fixture(
        "namespace-attribute-replaced-import",
        REPLACED_IMPORT_CONTRACT,
    );
    write(&root, "Cargo.toml", REPLACEMENT_WORKSPACE_MANIFEST);
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\npub struct Benign;\nimpl Benign { pub fn new(_: &str) -> Self { Self } }\nmod benign { pub use crate::Benign as Spawn; }\n#[macros::replace]\nuse benign::Spawn;\npub fn allowed() { let _ = Spawn::new(\"sh\"); }\n",
    );
    write_executor(&root);
    write_lock(&root);

    assert_missing_direct_owner(&check(&root), "attribute-replaced-import");
    reset(&root);
}

#[test]
fn attribute_replaced_benign_module_fails_closed() {
    let root = proc_macro_fixture(
        "namespace-attribute-replaced-module",
        REPLACED_MODULE_CONTRACT,
    );
    write(&root, "Cargo.toml", REPLACEMENT_WORKSPACE_MANIFEST);
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\npub struct Benign;\nimpl Benign { pub fn new(_: &str) -> Self { Self } }\n#[macros::replace_mod]\nmod bridge { pub use crate::Benign as Spawn; }\npub fn allowed() { let _ = bridge::Spawn::new(\"sh\"); }\n",
    );
    write_executor(&root);
    write_lock(&root);

    assert_unresolved_direct_owner(&check(&root), "attribute-replaced-module");
    reset(&root);
}

fn proc_macro_fixture(name: &str, contract: &str) -> std::path::PathBuf {
    let root = fixture(name, contract);
    std::fs::create_dir_all(root.join("emit_import/src")).expect("create proc-macro fixture");
    write(&root, "Cargo.toml", WORKSPACE_MANIFEST);
    write(&root, "emit_import/Cargo.toml", PROC_MACRO_MANIFEST);
    write(&root, "emit_import/src/lib.rs", PROC_MACRO_SOURCE);
    root
}

fn assert_unresolved_direct_owner(report: &Report, rule: &str) {
    assert!(
        report.findings.iter().any(|finding| {
            finding.path.as_deref() == Some("src/lib.rs")
                && finding.id == "OWN-005"
                && finding.rule == rule
                && finding.analysis == AnalysisQuality::Unresolved
        }),
        "{}",
        report.human()
    );
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.id.starts_with("RUST-MACRO-")),
        "{}",
        report.human()
    );
}

fn assert_missing_direct_owner(report: &Report, rule: &str) {
    assert!(
        report.findings.iter().any(|finding| {
            finding.path.as_deref() == Some("src/lib.rs")
                && finding.id == "OWN-004"
                && finding.rule == rule
        }),
        "{}",
        report.human()
    );
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.id.starts_with("RUST-MACRO-")),
        "{}",
        report.human()
    );
}

const WORKSPACE_MANIFEST: &str = r#"[workspace]
members = [".", "emit_import"]
resolver = "3"

[package]
name = "fixture"
version = "0.0.0"
edition = "2024"

[dependencies]
emit_import = { path = "emit_import" }
"#;

const REPLACEMENT_WORKSPACE_MANIFEST: &str = r#"[workspace]
members = [".", "emit_import"]
resolver = "3"

[package]
name = "fixture"
version = "0.0.0"
edition = "2024"

[dependencies]
macros = { package = "emit_import", path = "emit_import" }
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

#[proc_macro_attribute]
pub fn replace(_: TokenStream, _: TokenStream) -> TokenStream {
    "use std::process::Command as Spawn;".parse().unwrap()
}

#[proc_macro_attribute]
pub fn replace_mod(_: TokenStream, _: TokenStream) -> TokenStream {
    "mod bridge { pub use std::process::Command as Spawn; }".parse().unwrap()
}
"#;

const ATTRIBUTE_CONTRACT: &str = r#"
[[owner]]
name = "attribute-macro-import"
kind = "call"
within = ["src/**"]
match = "Spawn::new"
allow = ["src/lib.rs"]
reason = "Attribute-generated imports cannot receive exact direct-call authority."
"#;

const DERIVE_CONTRACT: &str = r#"
[[owner]]
name = "derive-macro-import"
kind = "call"
within = ["src/**"]
match = "Spawn::new"
allow = ["src/lib.rs"]
reason = "Derive-generated imports cannot receive exact direct-call authority."
"#;

const REPLACED_IMPORT_CONTRACT: &str = r#"
[[owner]]
name = "attribute-replaced-import"
kind = "call"
within = ["src/**"]
match = "Spawn::new"
allow = ["src/lib.rs"]
reason = "Replaced imports cannot lend their syntactic target exact authority."
"#;

const REPLACED_MODULE_CONTRACT: &str = r#"
[[owner]]
name = "attribute-replaced-module"
kind = "call"
within = ["src/**"]
match = "bridge::Spawn::new"
allow = ["src/lib.rs"]
reason = "Replaced modules cannot lend their syntactic body exact authority."
"#;
