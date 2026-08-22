//! Rejected macro authority is one deterministic invocation-level diagnostic.

use std::{fs, path::Path};

use zrail_rust::{build_lock, check_repository};

#[test]
fn unresolved_exact_allowance_is_attempted_and_suggests_conservative_binding() {
    let root = repository(
        "unresolved",
        MANIFEST,
        "//! Fixture.\npub fn run() { unknown!(); }\n",
        &allowance("unknown", ""),
    );

    let report = check(&root);
    let binding = only_binding_failure(&report);

    assert_eq!(
        count(&report, "RUST-MACRO-002"),
        0,
        "{:#?}",
        report.findings
    );
    assert!(
        binding
            .help
            .as_deref()
            .is_some_and(|help| help.contains(r#"binding = "conservative""#))
    );
    reset(&root);
}

#[test]
fn source_mismatch_reports_expected_and_observed_sources_once() {
    let manifest = format!(
        "{MANIFEST}\n[dependencies]\nreviewed_quote = {{ package = \"quote\", version = \"1\" }}\n"
    );
    let authority = r#"[source.rust.macros.allow.source]
kind = "git"
repository = "https://example.invalid/quote"
rev = "reviewed"
"#;
    let root = repository(
        "source",
        &manifest,
        "//! Fixture.\npub fn run() { reviewed_quote::quote!(); }\n",
        &allowance("quote::quote", authority),
    );

    let first = check(&root);
    let second = check(&root);
    let binding = only_binding_failure(&first);

    assert_eq!(first.findings, second.findings);
    assert!(
        binding.message.contains(
            "expects source \"git:https://example.invalid/quote:branch=:tag=:rev=reviewed:version=\""
        ),
        "{}",
        binding.message
    );
    assert!(
        binding
            .message
            .contains("quote@registry:crates-io:default-index:1"),
        "{}",
        binding.message
    );
    assert_eq!(count(&first, "RUST-MACRO-002"), 0, "{:#?}", first.findings);
    reset(&root);
}

#[test]
fn definition_mismatch_reports_configured_path_and_observed_package_once() {
    let source = r"//! Fixture.
mod support {
    macro_rules! reviewed { () => { 1 }; }
    pub(crate) use reviewed;
}
pub fn run() { let _ = support::reviewed!(); }
";
    let root = repository(
        "definition",
        MANIFEST,
        source,
        &allowance("support::reviewed", "definition = \"src/other.rs\""),
    );

    let report = check(&root);
    let binding = only_binding_failure(&report);

    assert!(binding.message.contains("src/other.rs"));
    assert!(binding.message.contains("fixture"));
    for id in ["RUST-MACRO-002", "RUST-MACRO-005"] {
        assert_eq!(count(&report, id), 0, "{:#?}", report.findings);
    }
    reset(&root);
}

#[test]
fn partial_candidate_attempt_is_rejected_without_unreviewed_or_stale_noise() {
    let source = r"//! Fixture.
mod one { macro_rules! reviewed { () => { 1 }; } pub(crate) use reviewed; }
mod two { macro_rules! reviewed { () => { 2 }; } pub(crate) use reviewed; }
mod tests {
    use super::{one::*, two::*};
    pub fn run() { reviewed!(); }
}
";
    let root = repository(
        "partial",
        MANIFEST,
        source,
        &allowance("super::one::reviewed", ""),
    );

    let report = check(&root);
    let binding = only_binding_failure(&report);

    assert!(binding.message.contains("super::two::reviewed"));
    for id in ["RUST-MACRO-001", "RUST-MACRO-002"] {
        assert_eq!(count(&report, id), 0, "{:#?}", report.findings);
    }
    reset(&root);
}

#[test]
fn allowance_without_a_syntactic_attempt_remains_stale() {
    let root = repository(
        "stale",
        MANIFEST,
        "//! Fixture.\npub fn run() {}\n",
        &allowance("unused", ""),
    );

    let report = check(&root);

    assert_eq!(
        count(&report, "RUST-MACRO-002"),
        1,
        "{:#?}",
        report.findings
    );
    assert_eq!(
        count(&report, "RUST-MACRO-006"),
        0,
        "{:#?}",
        report.findings
    );
    reset(&root);
}

fn only_binding_failure(report: &zrail_core::Report) -> &zrail_core::Finding {
    let bindings = report
        .findings
        .iter()
        .filter(|finding| finding.id == "RUST-MACRO-006")
        .collect::<Vec<_>>();
    assert_eq!(bindings.len(), 1, "{:#?}", report.findings);
    bindings[0]
}

fn count(report: &zrail_core::Report, id: &str) -> usize {
    report
        .findings
        .iter()
        .filter(|finding| finding.id == id)
        .count()
}

fn allowance(name: &str, authority: &str) -> String {
    format!(
        "[[source.rust.macros.allow]]\nname = \"{name}\"\nreason = \"Reviewed expansion boundary.\"\n{authority}\n"
    )
}

fn repository(name: &str, manifest: &str, source: &str, allowance: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!(
        "zrail-macro-binding-diagnostics-{name}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create fixture");
    fs::write(root.join("Cargo.toml"), manifest).expect("write manifest");
    fs::write(root.join("src/lib.rs"), source).expect("write source");
    fs::write(root.join("zrail.toml"), format!("{CONTRACT}\n{allowance}")).expect("write contract");
    build_lock(&root, "zrail.toml".as_ref())
        .expect("build fixture lock")
        .write(&root.join("zrail.lock"))
        .expect("write fixture lock");
    root
}

fn check(root: &Path) -> zrail_core::Report {
    check_repository(root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check fixture")
        .report
}

fn reset(root: &Path) {
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
