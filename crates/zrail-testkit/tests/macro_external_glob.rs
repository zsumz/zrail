//! Unknown external export sets accept only explicit conservative authority.

use std::{fs, path::Path};

use zrail_core::ReportStatus;
use zrail_rust::{build_lock, check_repository};

#[test]
fn external_glob_requires_and_accepts_conservative_source_authority() {
    let root = std::env::temp_dir().join(format!(
        "zrail-external-macro-glob-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create source fixture");
    write(&root, "Cargo.toml", MANIFEST);
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nuse external::prelude::*;\npub fn run() { reviewed!(); }\n",
    );
    write(&root, "zrail.toml", &contract("exact"));
    build_lock(&root, "zrail.toml".as_ref())
        .expect("build fixture lock")
        .write(&root.join("zrail.lock"))
        .expect("write fixture lock");

    let rejected = check(&root);
    let bindings = rejected
        .findings
        .iter()
        .filter(|finding| finding.id == "RUST-MACRO-006")
        .collect::<Vec<_>>();
    assert_eq!(bindings.len(), 1, "{}", rejected.human());
    assert!(bindings[0].message.contains("unknown export set"));
    assert!(
        bindings[0]
            .help
            .as_deref()
            .is_some_and(|help| help.contains(r#"resolution = "conservative""#))
    );

    write(&root, "zrail.toml", &contract("conservative"));
    build_lock(&root, "zrail.toml".as_ref())
        .expect("rebuild fixture lock")
        .write(&root.join("zrail.lock"))
        .expect("rewrite fixture lock");
    let report = check(&root);

    assert_eq!(report.status, ReportStatus::Pass, "{}", report.human());

    write(
        &root,
        "src/lib.rs",
        "//! Library.\nuse external::prelude::*;\nuse external::reviewed;\npub fn run() { reviewed!(); }\n",
    );
    build_lock(&root, "zrail.toml".as_ref())
        .expect("rebuild mixed-candidate fixture lock")
        .write(&root.join("zrail.lock"))
        .expect("rewrite mixed-candidate fixture lock");
    let mixed = check(&root);

    assert_eq!(mixed.status, ReportStatus::Pass, "{}", mixed.human());
    reset(&root);
}

fn check(root: &Path) -> zrail_core::Report {
    check_repository(root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check external macro glob")
        .report
}

fn write(root: &Path, path: &str, contents: &str) {
    fs::write(root.join(path), contents).expect("write fixture");
}

fn reset(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).expect("reset fixture");
    }
}

const MANIFEST: &str = r#"[package]
name = "fixture"
version = "0.0.0"
edition = "2024"
[dependencies]
external = "1"
"#;

fn contract(resolution: &str) -> String {
    CONTRACT.replace("RESOLUTION", resolution)
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
[[dependencies.crate_root]]
package = "external"
root = "external"
reason = "The published library exposes the external crate root."
[dependencies.crate_root.source]
kind = "registry"
requirement = "1"
[source.rust]
module_docs = "required"
facades = "allow"
tests = "allow"
[source.rust.macros]
mode = "deny-unreviewed"
[[source.rust.macros.allow]]
name = "reviewed"
resolution = "RESOLUTION"
reason = "Reviewed unresolved external prelude macro."
[source.rust.macros.allow.source]
kind = "registry"
requirement = "1"
[source.rust.hygiene]
unsafe = "deny"
lint_suppressions = "allow"
"#;
