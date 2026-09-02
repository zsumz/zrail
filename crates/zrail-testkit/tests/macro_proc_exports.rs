//! Proc-macro crate roots contribute their declared macro exports to globs.

use std::{fs, path::Path};

use zrail_core::ReportStatus;
use zrail_rust::{build_lock, check_repository};

#[test]
fn proc_macro_export_resolves_through_a_dependency_glob() {
    let root = std::env::temp_dir().join(format!(
        "zrail-proc-macro-export-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    for package in ["macro_impl", "consumer"] {
        fs::create_dir_all(root.join(format!("crates/{package}/src")))
            .expect("create package fixture");
    }
    write(&root, "Cargo.toml", WORKSPACE_MANIFEST);
    write(
        &root,
        "crates/macro_impl/Cargo.toml",
        r#"[package]
name = "macro-impl"
version = "0.0.0"
edition = "2024"
[lib]
proc-macro = true
"#,
    );
    write(
        &root,
        "crates/macro_impl/src/lib.rs",
        r"//! Macro implementation.
extern crate proc_macro;
use proc_macro::TokenStream;

#[proc_macro]
pub fn reviewed(input: TokenStream) -> TokenStream { input }
",
    );
    write(
        &root,
        "crates/consumer/Cargo.toml",
        r#"[package]
name = "consumer"
version = "0.0.0"
edition = "2024"
[dependencies]
macro_impl = { package = "macro-impl", path = "../macro_impl" }
"#,
    );
    write(
        &root,
        "crates/consumer/src/lib.rs",
        "//! Macro consumer.\nuse macro_impl::*;\npub fn run() { reviewed!(); }\n",
    );
    write(&root, "zrail.toml", CONTRACT);
    build_lock(&root, "zrail.toml".as_ref())
        .expect("build fixture lock")
        .write(&root.join("zrail.lock"))
        .expect("write fixture lock");

    let report = check_repository(&root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check proc-macro export")
        .report;

    assert_eq!(report.status, ReportStatus::Pass, "{}", report.human());
    reset(&root);
}

fn write(root: &Path, path: &str, contents: &str) {
    fs::write(root.join(path), contents).expect("write fixture");
}

fn reset(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).expect("reset fixture");
    }
}

const WORKSPACE_MANIFEST: &str = "[workspace]\nmembers = [\"crates/*\"]\nresolver = \"3\"\n";

const CONTRACT: &str = r#"schema = 1
adapters = ["rust"]
[repository]
roots = ["crates"]
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
name = "macro_impl::reviewed"
definition = "crates/macro_impl/src/lib.rs"
reason = "Reviewed repository proc-macro expansion."
[source.rust.hygiene]
unsafe = "deny"
lint_suppressions = "allow"
"#;
