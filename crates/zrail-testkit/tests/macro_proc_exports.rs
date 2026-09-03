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

#[test]
fn proc_macro_export_resolves_through_a_named_path_dependency_reexport() {
    let root = std::env::temp_dir().join(format!(
        "zrail-proc-macro-reexport-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    for package in ["macro_impl", "bridge", "consumer"] {
        fs::create_dir_all(root.join(format!("crates/{package}/src")))
            .expect("create package fixture");
    }
    fs::create_dir_all(root.join("crates/consumer/src/tests")).expect("create nested test fixture");
    fs::create_dir_all(root.join("crates/bridge/tests")).expect("create integration test fixture");
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

#[proc_macro_attribute]
pub fn reviewed(_: TokenStream, item: TokenStream) -> TokenStream { item }
",
    );
    write(
        &root,
        "crates/bridge/Cargo.toml",
        &package_manifest(
            "bridge",
            "macro_impl = { package = \"macro-impl\", path = \"../macro_impl\" }\n",
        ),
    );
    write(
        &root,
        "crates/bridge/src/lib.rs",
        "//! Macro bridge.\npub use macro_impl::reviewed;\n",
    );
    write(
        &root,
        "crates/bridge/tests/api.rs",
        "//! Public API check.\nuse bridge::reviewed;\n#[reviewed]\nfn api() {}\n",
    );
    write(
        &root,
        "crates/consumer/Cargo.toml",
        r#"[package]
name = "consumer"
version = "0.0.0"
edition = "2024"
[dev-dependencies]
bridge = { path = "../bridge" }
"#,
    );
    write(
        &root,
        "crates/consumer/src/lib.rs",
        "//! Macro consumer.\n#[cfg(test)]\nmod tests;\n",
    );
    write(
        &root,
        "crates/consumer/src/tests.rs",
        "//! Test namespace.\nuse super::*;\nmod nested;\n",
    );
    write(
        &root,
        "crates/consumer/src/tests/nested.rs",
        "//! Nested macro consumer.\n#[::bridge::reviewed]\nfn run() {}\n",
    );
    write(&root, "zrail.toml", REEXPORT_CONTRACT);
    let lock = build_lock(&root, "zrail.toml".as_ref()).expect("build fixture lock");
    assert_eq!(lock.macro_implementations.len(), 1);
    assert_eq!(lock.macro_implementations[0].package, "bridge");
    assert_eq!(lock.macro_implementations[0].directory, "crates/bridge");
    lock.write(&root.join("zrail.lock"))
        .expect("write fixture lock");

    let report = check_repository(&root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check proc-macro re-export")
        .report;

    assert_eq!(report.status, ReportStatus::Pass, "{}", report.human());

    write(
        &root,
        "crates/macro_impl/src/lib.rs",
        r"//! Changed macro implementation.
extern crate proc_macro;
use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn reviewed(_: TokenStream, item: TokenStream) -> TokenStream {
    let _changed = true;
    item
}
",
    );
    let changed = check_repository(&root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check changed proc-macro implementation")
        .report;
    assert!(
        changed
            .findings
            .iter()
            .any(|finding| { finding.id == "LOCK-023" && finding.message.contains("bridge") }),
        "{}",
        changed.human()
    );
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

fn package_manifest(name: &str, dependencies: &str) -> String {
    format!(
        "[package]\nname = \"{name}\"\nversion = \"0.0.0\"\nedition = \"2024\"\n[dependencies]\n{dependencies}"
    )
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

const REEXPORT_CONTRACT: &str = r#"schema = 1
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
name = "bridge::reviewed"
reason = "Reviewed repository proc-macro re-export."
[source.rust.macros.allow.source]
kind = "repository"
package = "bridge"
directory = "crates/bridge"
ambient_inputs = "none"
[source.rust.hygiene]
unsafe = "deny"
lint_suppressions = "allow"
"#;
