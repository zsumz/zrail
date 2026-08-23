//! Macro visibility consumes the exact module edges selected by the source graph.

use std::{fs, path::Path};

use zrail_core::ReportStatus;
use zrail_rust::{build_lock, check_repository};

#[test]
fn direct_module_child_ignores_an_unselected_source_parent_candidate() {
    let root = std::env::temp_dir().join(format!(
        "zrail-macro-module-edges-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    fs::create_dir_all(root.join("src/foo")).expect("create fixture directories");
    write(&root, "Cargo.toml", MANIFEST);
    write(&root, "src/lib.rs", "//! Library.\nmod foo;\nmod child;\n");
    write(&root, "src/child.rs", "//! Root child.\npub fn run() {}\n");
    write(
        &root,
        "src/foo.rs",
        "//! Direct module.\nmacro_rules! reviewed { () => {}; }\npub(crate) use reviewed;\nmod child;\n",
    );
    write(
        &root,
        "src/foo/child.rs",
        "//! Nested child.\nuse super::*;\npub fn run() { reviewed!(); }\n",
    );
    write(&root, "zrail.toml", CONTRACT);
    build_lock(&root, "zrail.toml".as_ref())
        .expect("build fixture lock")
        .write(&root.join("zrail.lock"))
        .expect("write fixture lock");

    let report = check_repository(&root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check exact module visibility")
        .report;

    assert_eq!(report.status, ReportStatus::Pass, "{}", report.human());
    reset(&root);
}

#[test]
fn bare_local_module_glob_does_not_claim_unexported_macros() {
    let root = std::env::temp_dir().join(format!(
        "zrail-macro-local-module-glob-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create fixture directories");
    write(&root, "Cargo.toml", MANIFEST);
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod support;\nuse support::*;\n#[derive(Clone)]\npub struct Model;\n",
    );
    write(
        &root,
        "src/support.rs",
        "//! Test support.\npub struct Helper;\n",
    );
    write(
        &root,
        "zrail.toml",
        &CONTRACT
            .replace("super::reviewed", "Clone")
            .replace("definition = \"src/foo.rs\"\n", ""),
    );
    build_lock(&root, "zrail.toml".as_ref())
        .expect("build fixture lock")
        .write(&root.join("zrail.lock"))
        .expect("write fixture lock");

    let report = check_repository(&root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check local module glob visibility")
        .report;

    assert_eq!(report.status, ReportStatus::Pass, "{}", report.human());
    reset(&root);
}

#[test]
fn test_only_module_edge_cannot_authorize_a_production_macro() {
    let root = std::env::temp_dir().join(format!(
        "zrail-macro-test-edge-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create fixture directories");
    write(
        &root,
        "Cargo.toml",
        &format!(
            "{MANIFEST}[dependencies]\nreviewed_json = {{ package = \"serde_json\", version = \"1\" }}\n"
        ),
    );
    write(
        &root,
        "src/lib.rs",
        "//! Library.\n#[cfg(test)] mod support;\nuse support::*;\npub fn run() { let _ = reviewed!({\"ok\": true}); }\n",
    );
    write(
        &root,
        "src/support.rs",
        "//! Test support.\npub use reviewed_json::json as reviewed;\n",
    );
    write(&root, "zrail.toml", EXTERNAL_CONTRACT);

    let report = check_repository(&root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check test-only macro visibility")
        .report;

    assert!(report.findings.iter().any(|finding| {
        finding.id.starts_with("RUST-MACRO") && finding.message.contains("reviewed")
    }));
    reset(&root);
}

#[test]
fn test_only_module_edge_can_authorize_a_test_only_macro() {
    let root = std::env::temp_dir().join(format!(
        "zrail-macro-test-edge-ok-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create fixture directories");
    write(
        &root,
        "Cargo.toml",
        &format!(
            "{MANIFEST}[dependencies]\nreviewed_json = {{ package = \"serde_json\", version = \"1\" }}\n"
        ),
    );
    write(
        &root,
        "src/lib.rs",
        "//! Library.\n#[cfg(test)] mod support;\n#[cfg(test)] use support::*;\n#[cfg(test)] pub fn run() { let _ = reviewed!({\"ok\": true}); }\n",
    );
    write(
        &root,
        "src/support.rs",
        "//! Test support.\npub use reviewed_json::json as reviewed;\n",
    );
    write(&root, "zrail.toml", EXTERNAL_CONTRACT);

    let report = check_repository(&root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check test-only macro visibility")
        .report;

    assert!(!report.findings.iter().any(|finding| {
        finding.id.starts_with("RUST-MACRO") && finding.message.contains("reviewed")
    }));
    reset(&root);
}

fn write(root: &Path, path: &str, contents: &str) {
    fs::write(root.join(path), contents).expect("write fixture file");
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

[[source.rust.macros.allow]]
name = "super::reviewed"
definition = "src/foo.rs"
reason = "Reviewed repository macro expansion."

[source.rust.hygiene]
unsafe = "deny"
lint_suppressions = "allow"
deny_methods = []
deny_macros = []
"#;

const EXTERNAL_CONTRACT: &str = r#"schema = 1
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
name = "serde_json::json"
inputs = "opaque"
reason = "Reviewed external expansion."
[source.rust.macros.allow.source]
kind = "registry"
requirement = "1"
[source.rust.hygiene]
unsafe = "deny"
lint_suppressions = "allow"
deny_methods = []
deny_macros = []
"#;
