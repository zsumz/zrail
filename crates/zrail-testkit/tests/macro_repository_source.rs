//! Cross-crate repository macro claims bind exact package provenance.

use std::{fs, path::Path};

use zrail_core::ReportStatus;
use zrail_rust::{build_lock, check_repository};

#[test]
fn repository_source_closes_only_the_matching_workspace_macro() {
    let root = fixture("matching", "macros");
    build_lock(&root, "zrail.toml".as_ref())
        .expect("build repository-source lock")
        .write(&root.join("zrail.lock"))
        .expect("write repository-source lock");

    let report = check_repository(&root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check repository macro source")
        .report;

    assert_eq!(report.status, ReportStatus::Pass, "{}", report.human());
    reset(&root);
}

#[test]
fn repository_source_rejects_a_different_package_directory() {
    let root = fixture("mismatch", "other-macros");
    let report = check_repository(&root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check mismatched repository macro source")
        .report;

    assert!(
        report.findings.iter().any(|finding| {
            finding.id == "RUST-MACRO-006" && finding.message.contains("repository")
        }),
        "{}",
        report.human()
    );
    reset(&root);
}

fn fixture(name: &str, source_directory: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!(
        "zrail-repository-macro-{name}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    fs::create_dir_all(root.join("app/src")).expect("create app");
    fs::create_dir_all(root.join("macros/src")).expect("create macro package");
    write(
        &root,
        "Cargo.toml",
        "[workspace]\nmembers = [\"app\", \"macros\"]\nresolver = \"3\"\n",
    );
    write(
        &root,
        "app/Cargo.toml",
        "[package]\nname = \"app\"\nversion = \"0.0.0\"\nedition = \"2024\"\n[dependencies]\nworkspace-macros = { path = \"../macros\" }\n",
    );
    write(
        &root,
        "macros/Cargo.toml",
        "[package]\nname = \"workspace-macros\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
    );
    write(
        &root,
        "app/src/lib.rs",
        "//! Consumer.\nmod owner;\npub struct State { pub epoch: usize }\npub fn run() { workspace_macros::reviewed!(); }\n",
    );
    write(
        &root,
        "app/src/owner.rs",
        "//! State mutation owner.\nuse crate::State;\npub fn advance(state: &mut State) { state.epoch += 1; }\n",
    );
    write(
        &root,
        "macros/src/lib.rs",
        "//! Macro package.\n#[macro_export]\nmacro_rules! reviewed { () => {}; }\n",
    );
    write(
        &root,
        "zrail.toml",
        &CONTRACT.replace("SOURCE_DIRECTORY", source_directory),
    );
    root
}

fn write(root: &Path, relative: &str, contents: &str) {
    fs::write(root.join(relative), contents).expect("write fixture");
}

fn reset(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).expect("reset fixture");
    }
}

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
[[source.rust.macros.allow]]
name = "workspace_macros::reviewed"
field_mutation = "none"
reason = "Reviewed workspace macro performs no field mutation."
[source.rust.macros.allow.source]
kind = "repository"
ambient_inputs = "none"
package = "workspace-macros"
directory = "SOURCE_DIRECTORY"
[source.rust.hygiene]
unsafe = "deny"
lint_suppressions = "allow"
[[owner]]
name = "state-write"
kind = "field-write"
within = ["app/src/**"]
match = "crate::State::epoch"
allow = ["app/src/owner.rs"]
reason = "State writes stay in the owner module."
"#;
