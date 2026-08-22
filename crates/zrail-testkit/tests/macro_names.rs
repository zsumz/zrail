//! Macro authority names remain stable, user-spellable paths across lexical aliases.

use std::{fs, path::Path};

use zrail_core::ReportStatus;
use zrail_rust::{build_lock, check_repository};

#[test]
fn dependency_import_forms_share_one_public_policy_name() {
    let root = repository(
        "dependency-imports",
        r"//! Dependency macro names.
use reviewed_quote::quote;
use reviewed_quote::quote as q;
use reviewed_quote::quote as r#async;
pub use reviewed_quote::quote as exported;
pub fn run() { let _ = quote!(); let _ = q!(); let _ = r#async!(); let _ = exported!(); let _ = reviewed_quote::quote!(); }
",
        &external_allowance("quote::quote"),
        "reviewed_quote = { package = \"quote\", version = \"1\" }",
    );
    lock(&root);

    let report = check(&root);

    assert_eq!(report.status, ReportStatus::Pass, "{:#?}", report.findings);
    reset(&root);
}

#[test]
fn exact_written_alias_is_accepted_without_becoming_the_preferred_name() {
    let root = repository(
        "written-alias",
        "//! Renamed macro.\nuse reviewed_quote::quote as q;\npub fn run() { let _ = q!(); }\n",
        &external_allowance("q"),
        "reviewed_quote = { package = \"quote\", version = \"1\" }",
    );
    lock(&root);

    let report = check(&root);

    assert_eq!(report.status, ReportStatus::Pass, "{:#?}", report.findings);
    reset(&root);
}

#[test]
fn unreviewed_same_named_import_suggests_one_valid_public_path() {
    let root = repository(
        "public-suggestion",
        "//! Same-named macro.\nuse quote::quote;\npub fn run() { let _ = quote!(); }\n",
        "",
        "quote = \"1\"",
    );
    lock(&root);

    let report = check(&root);
    let finding = report
        .findings
        .iter()
        .find(|finding| finding.id == "RUST-MACRO-001")
        .expect("unreviewed macro finding");

    assert!(
        finding
            .message
            .contains("preferred policy name quote::quote")
    );
    assert!(
        finding
            .help
            .as_deref()
            .is_some_and(|help| help.contains("name = \"quote::quote\""))
    );
    assert!(!finding.message.contains("quote::quote::quote"));
    reset(&root);
}

#[test]
fn repository_reexport_accepts_public_and_written_spellings() {
    let source = r"//! Repository macro names.
mod local {
    macro_rules! reviewed { () => { 1 }; }
    pub(crate) use reviewed as exposed;
}
use local::exposed as local_alias;
pub fn run() { let _ = local_alias!(); }
";
    for name in ["local::exposed", "local_alias"] {
        let root = repository(
            name.replace("::", "-as-").as_str(),
            source,
            &allowance(name),
            "",
        );
        lock(&root);
        let report = check(&root);
        assert_eq!(report.status, ReportStatus::Pass, "{:#?}", report.findings);
        reset(&root);
    }
}

fn external_allowance(name: &str) -> String {
    format!(
        "{}[source.rust.macros.allow.source]\nkind = \"registry\"\nrequirement = \"1\"\n",
        allowance(name)
    )
}

fn allowance(name: &str) -> String {
    format!(
        "\n[[source.rust.macros.allow]]\nname = \"{name}\"\nreason = \"Reviewed macro expansion boundary.\"\n"
    )
}

fn repository(
    name: &str,
    source: &str,
    allowances: &str,
    dependencies: &str,
) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!(
        "zrail-macro-names-{name}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create source root");
    fs::write(
        root.join("Cargo.toml"),
        format!("{MANIFEST}\n[dependencies]\n{dependencies}\n"),
    )
    .expect("write manifest");
    fs::write(root.join("src/lib.rs"), source).expect("write source");
    fs::write(root.join("zrail.toml"), format!("{CONTRACT}{allowances}")).expect("write contract");
    root
}

fn lock(root: &Path) {
    build_lock(root, "zrail.toml".as_ref())
        .expect("build fixture lock")
        .write(&root.join("zrail.lock"))
        .expect("write fixture lock");
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
