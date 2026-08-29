//! Item-position workspace macros bind source identity and implementation content.

use std::{fs, path::Path};

use zrail_core::ReportStatus;
use zrail_rust::{build_lock, check_repository};

#[test]
fn matching_repository_source_binds_item_manifest_and_implementation() {
    let root = fixture("matching", "workspace-macros", "macros");
    let lock = build_lock(&root, "zrail.toml".as_ref()).expect("build item macro lock");

    let authority = lock
        .item_macro_manifests
        .first()
        .expect("locked item macro manifest");
    assert_eq!(authority.definition, "repository:workspace-macros:macros");
    assert_eq!(authority.definition_sha256.len(), 64);
    let implementation = lock
        .macro_implementations
        .first()
        .expect("locked macro implementation");
    assert_eq!(implementation.package, "workspace-macros");
    assert_eq!(implementation.directory, "macros");
    assert_eq!(implementation.manifest_sha256.len(), 64);
    assert_eq!(authority.definition_sha256, implementation.manifest_sha256);

    lock.write(&root.join("zrail.lock")).expect("write lock");
    let report = check_repository(&root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check matching item macro source")
        .report;
    assert_eq!(report.status, ReportStatus::Pass, "{}", report.human());
    reset(&root);
}

#[test]
fn repository_item_source_requires_exact_package_and_directory() {
    for (name, package, directory) in [
        ("package", "other-macros", "macros"),
        ("directory", "workspace-macros", "other-macros"),
    ] {
        let root = fixture(name, package, directory);
        let error = build_lock(&root, "zrail.toml".as_ref())
            .expect_err("mismatched repository source must fail closed");
        assert!(
            error
                .to_string()
                .contains("does not match repository source authority"),
            "{error}"
        );
        reset(&root);
    }
}

#[test]
fn repository_item_macro_content_changes_both_lock_surfaces() {
    let root = fixture("changed", "workspace-macros", "macros");
    build_lock(&root, "zrail.toml".as_ref())
        .expect("build item macro lock")
        .write(&root.join("zrail.lock"))
        .expect("write item macro lock");
    write(&root, "macros/src/lib.rs", CHANGED_MACRO_SOURCE);

    let report = check_repository(&root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check changed repository macro")
        .report;
    assert!(has(&report, "LOCK-023"), "{}", report.human());
    assert!(has(&report, "LOCK-035"), "{}", report.human());
    reset(&root);
}

#[test]
fn transitive_internal_helper_content_changes_both_lock_surfaces() {
    let root = fixture("helper-changed", "workspace-macros", "macros");
    build_lock(&root, "zrail.toml".as_ref())
        .expect("build item macro lock")
        .write(&root.join("zrail.lock"))
        .expect("write item macro lock");
    write(&root, "support/src/lib.rs", CHANGED_SUPPORT_SOURCE);

    let report = check_repository(&root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check changed transitive macro helper")
        .report;
    assert!(has(&report, "LOCK-023"), "{}", report.human());
    assert!(has(&report, "LOCK-035"), "{}", report.human());
    reset(&root);
}

fn fixture(name: &str, source_package: &str, source_directory: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!(
        "zrail-repository-item-macro-{name}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    for directory in [
        "app/src",
        "helper/src",
        "macros/src",
        "support/src",
        "zrail/macros",
    ] {
        fs::create_dir_all(root.join(directory)).expect("create fixture directory");
    }
    write(&root, "Cargo.toml", WORKSPACE);
    write(&root, "app/Cargo.toml", APP_PACKAGE);
    write(&root, "helper/Cargo.toml", HELPER_PACKAGE);
    write(&root, "macros/Cargo.toml", MACRO_PACKAGE);
    write(&root, "support/Cargo.toml", SUPPORT_PACKAGE);
    write(&root, "app/src/lib.rs", APP_SOURCE);
    write(&root, "helper/src/lib.rs", HELPER_SOURCE);
    write(&root, "macros/src/lib.rs", MACRO_SOURCE);
    write(&root, "support/src/lib.rs", SUPPORT_SOURCE);
    write(&root, "zrail/macros/declare.toml", ITEM_MANIFEST);
    write(
        &root,
        "zrail.toml",
        &CONTRACT
            .replace("SOURCE_PACKAGE", source_package)
            .replace("SOURCE_DIRECTORY", source_directory),
    );
    root
}

fn has(report: &zrail_core::Report, id: &str) -> bool {
    report.findings.iter().any(|finding| finding.id == id)
}

fn write(root: &Path, relative: &str, contents: &str) {
    fs::write(root.join(relative), contents).expect("write fixture");
}

fn reset(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).expect("reset fixture");
    }
}

const WORKSPACE: &str = "[workspace]\nmembers = [\"app\", \"helper\", \"macros\"]\nexclude = [\"support\"]\nresolver = \"3\"\n";
const APP_PACKAGE: &str = r#"[package]
name = "app"
version = "0.0.0"
edition = "2024"
[dependencies]
workspace-macros = { path = "../macros" }
"#;
const MACRO_PACKAGE: &str = r#"[package]
name = "workspace-macros"
version = "0.0.0"
edition = "2024"
[dependencies]
macro-helper = { path = "../helper" }
"#;
const HELPER_PACKAGE: &str = r#"[package]
name = "macro-helper"
version = "0.0.0"
edition = "2024"
[dependencies]
macro-support = { path = "../support" }
"#;
const SUPPORT_PACKAGE: &str =
    "[package]\nname = \"macro-support\"\nversion = \"0.0.0\"\nedition = \"2024\"\n";
const APP_SOURCE: &str =
    "//! Consumer.\nworkspace_macros::declare!();\nfn accepts_generated(_: Generated) {}\n";
const MACRO_SOURCE: &str =
    "//! Macro package.\n#[macro_export]\nmacro_rules! declare { () => { struct Generated; } }\n";
const CHANGED_MACRO_SOURCE: &str = "//! Macro package.\n#[macro_export]\nmacro_rules! declare { () => { struct Generated { value: usize } } }\n";
const HELPER_SOURCE: &str = "//! Macro helper.\npub const VALUE: usize = macro_support::VALUE;\n";
const SUPPORT_SOURCE: &str = "//! Macro support.\npub const VALUE: usize = 1;\n";
const CHANGED_SUPPORT_SOURCE: &str = "//! Macro support.\npub const VALUE: usize = 2;\n";
const ITEM_MANIFEST: &str = r#"schema = 1
macro_name = "workspace_macros::declare"
invocation_sha256 = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
[[binding]]
name = "Generated"
kind = "type"
public = false
"#;
const CONTRACT: &str = r#"schema = 2
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
[[source.rust.item_macros]]
name = "workspace_macros::declare"
path = "app/src/lib.rs"
resolution = "exact"
source = { kind = "repository", package = "SOURCE_PACKAGE", directory = "SOURCE_DIRECTORY" }
manifest = "zrail/macros/declare.toml"
reason = "The exact workspace macro owns this manifested namespace."
[source.rust.hygiene]
unsafe = "deny"
lint_suppressions = "allow"
"#;
