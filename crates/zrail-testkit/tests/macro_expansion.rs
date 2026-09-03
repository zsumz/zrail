//! Macro-generated Rust is denied unless its expansion boundary is explicitly reviewed.

use std::{fs, path::PathBuf};

use zrail_core::{Finding, ReportStatus};
use zrail_rust::{build_lock, check_repository};

#[path = "macro_expansion/binding_opacity.rs"]
mod binding_opacity;
#[path = "macro_expansion/definition_binding.rs"]
mod definition_binding;
#[path = "macro_expansion/intrinsic_shadow.rs"]
mod intrinsic_shadow;
#[path = "macro_expansion/staging.rs"]
mod staging;

#[test]
fn local_macro_cannot_hide_unsafe_code_or_process_effects() {
    let root = repository(
        "local-hidden",
        r#"//! Hidden expansion fixture.
macro_rules! hidden {
    () => {{ unsafe { core::ptr::read_volatile(&0) }; std::process::Command::new("sh") }};
}
pub fn run() { let _ = hidden!(); }
"#,
        "",
    );

    let report = check(&root);

    assert_finding(&report.findings, "RUST-MACRO-001", "hidden");
    reset(&root);
}

#[test]
fn expression_derive_and_attribute_macros_are_expansion_boundaries() {
    let root = repository(
        "macro-kinds",
        r#"//! Expansion kinds.
#[derive(CustomDerive)]
struct Message;

#[custom_attribute]
pub fn run() { format!("{}", std::process::Command::new("sh")); }
"#,
        "",
    );

    let report = check(&root);

    for name in ["CustomDerive", "custom_attribute", "format"] {
        assert_finding(&report.findings, "RUST-MACRO-001", name);
    }
    reset(&root);
}

#[test]
fn reviewed_expansion_is_allowed_and_stale_authority_is_rejected() {
    let root = repository(
        "allowed",
        "//! Reviewed expansion.\nmod local { macro_rules! reviewed { () => { 1 }; } pub(crate) use reviewed; }\npub fn run() { let _ = local::reviewed!(); }\n",
        r#"
[[source.rust.macros.allow]]
name = "local::reviewed"
definition = "src/lib.rs"
reason = "The local transcriber expands to one integer literal."
"#,
    );
    build_lock(&root, "zrail.toml".as_ref())
        .expect("build content-bound macro lock")
        .write(&root.join("zrail.lock"))
        .expect("write content-bound macro lock");

    let report = check(&root);
    assert_eq!(report.status, ReportStatus::Pass, "{:#?}", report.findings);
    fs::write(
        root.join("src/lib.rs"),
        "//! No expansion.\npub fn run() {}\n",
    )
    .expect("remove invocation");

    let stale = check(&root);
    assert_finding(&stale.findings, "RUST-MACRO-002", "local::reviewed");
    reset(&root);
}

#[test]
fn include_expansion_is_inspected_instead_of_blanket_trusted() {
    let root = repository(
        "include",
        "//! Inspected include.\ninclude!(\"support.rs\");\n",
        "",
    );
    fs::write(
        root.join("src/support.rs"),
        "//! Included source.\npub fn run() { unsafe { core::ptr::read_volatile(&0); } }\n",
    )
    .expect("write included source");

    let report = check(&root);

    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.id == "RUST-HYG-004")
    );
    assert!(
        !report.findings.iter().any(|finding| {
            finding.id == "RUST-MACRO-001" && finding.message.contains("include")
        })
    );
    reset(&root);
}

#[test]
fn lexical_alias_authority_does_not_leak_to_same_spelled_macros() {
    let root = repository(
        "alias-scope",
        r"//! Scoped alias fixture.
mod rt { macro_rules! select { () => { unsafe { core::ptr::read_volatile(&0) } }; } }
mod reviewed_module { use tokio as runtime; pub fn run() { runtime::select! {} } }
pub fn reviewed() { use tokio as rt; rt::select! {} }
pub fn hidden() { rt::select! {} }
",
        r#"
[[source.rust.macros.allow]]
name = "tokio::select"
reason = "Only the invocation beneath the exact lexical import is reviewed."
[source.rust.macros.allow.source]
kind = "registry"
requirement = "1"
"#,
    );
    fs::write(
        root.join("Cargo.toml"),
        format!("{MANIFEST}\n[dependencies]\ntokio = {{ package = \"tokio\", version = \"1\" }}\n"),
    )
    .expect("add aliased dependency");
    build_lock(&root, "zrail.toml".as_ref())
        .expect("build lexical macro lock")
        .write(&root.join("zrail.lock"))
        .expect("write lexical macro lock");

    let report = check(&root);
    let boundaries = report
        .findings
        .iter()
        .filter(|finding| finding.id == "RUST-MACRO-001")
        .collect::<Vec<_>>();
    assert_eq!(boundaries.len(), 1, "{:#?}", report.findings);
    assert!(boundaries[0].message.contains("rt::select"));
    reset(&root);
}

#[test]
fn conditional_import_cannot_authorize_macro_identity() {
    let root = repository(
        "conditional-alias",
        concat!(
            "//! Conditional aliases.\n",
            "#[cfg(any())] use tokio as top;\n",
            "pub fn top_level() { top::select! {} }\n",
            "pub fn local() { #[cfg(any())] use tokio as local; local::select! {} }\n",
        ),
        r#"
[[source.rust.macros.allow]]
name = "tokio::select"
reason = "The runtime macro is reviewed only when its identity is exact."
[source.rust.macros.allow.source]
kind = "registry"
requirement = "1"
"#,
    );
    fs::write(
        root.join("Cargo.toml"),
        format!("{MANIFEST}\n[dependencies]\ntokio = {{ package = \"tokio\", version = \"1\" }}\n"),
    )
    .expect("add aliased dependency");
    build_lock(&root, "zrail.toml".as_ref())
        .expect("build conditional macro lock")
        .write(&root.join("zrail.lock"))
        .expect("write conditional macro lock");

    let report = check(&root);
    let boundaries = report
        .findings
        .iter()
        .filter(|finding| finding.id == "RUST-MACRO-001")
        .collect::<Vec<_>>();
    assert_eq!(boundaries.len(), 2, "{:#?}", report.findings);
    assert!(boundaries.iter().any(|finding| {
        finding.message.contains("top::select")
            && finding.analysis == zrail_core::AnalysisQuality::Unresolved
    }));
    assert!(boundaries.iter().any(|finding| {
        finding.message.contains("local::select")
            && finding.analysis == zrail_core::AnalysisQuality::Exact
    }));
    assert!(report.findings.iter().any(|finding| {
        finding.id == "RUST-MACRO-002" && finding.message.contains("tokio::select")
    }));
    reset(&root);
}

#[test]
fn malformed_expansion_attributes_fail_closed() {
    let root = repository(
        "malformed-attribute",
        "//! Malformed expansion.\n#[derive(name = value)]\npub struct Message;\n",
        "",
    );

    let report = check(&root);
    assert!(report.findings.iter().any(|finding| {
        finding.id == "RUST-MACRO-001"
            && finding.analysis == zrail_core::AnalysisQuality::Unresolved
    }));
    reset(&root);
}

fn assert_finding(findings: &[Finding], id: &str, name: &str) {
    assert!(
        findings
            .iter()
            .any(|finding| finding.id == id && finding.message.contains(name)),
        "missing {id} for {name}: {findings:#?}"
    );
}

fn check(root: &std::path::Path) -> zrail_core::Report {
    check_repository(root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("analyze macro fixture")
        .report
}

fn repository(name: &str, source: &str, allowances: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "zrail-macro-{name}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create source root");
    fs::write(root.join("Cargo.toml"), MANIFEST).expect("write manifest");
    fs::write(root.join("src/lib.rs"), source).expect("write source");
    fs::write(root.join("zrail.toml"), format!("{CONTRACT}{allowances}")).expect("write contract");
    root
}

fn reset(root: &PathBuf) {
    if root.exists() {
        fs::remove_dir_all(root).expect("reset fixture");
    }
}

const MANIFEST: &str = "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n";

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
mode = "deny-unreviewed"

[source.rust.hygiene]
unsafe = "deny"
lint_suppressions = "allow"
deny_methods = []
deny_macros = []
"#;
