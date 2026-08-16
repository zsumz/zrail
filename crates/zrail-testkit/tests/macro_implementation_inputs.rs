//! Repository macro manifests bind Cargo metadata and literal compile-time inputs.

use std::{
    fs,
    path::{Path, PathBuf},
};

use zrail_rust::{build_lock, check_repository};

#[test]
fn included_non_rust_input_changes_invalidate_macro_authority() {
    let root = repository("included-input");
    write(&root, "src/template.txt", "safe template\n");
    write(
        &root,
        "src/lib.rs",
        r#"//! Macro implementation inputs.
const TEMPLATE: &str = include_str!("template.txt");
mod helpers {
    macro_rules! reviewed { () => { TEMPLATE }; }
    pub(crate) use reviewed;
}
pub fn run() { let _ = helpers::reviewed!(); }
"#,
    );
    lock(&root);
    assert!(
        !check(&root)
            .findings
            .iter()
            .any(|finding| finding.id.starts_with("LOCK-"))
    );

    write(
        &root,
        "Cargo.toml",
        &format!("{MANIFEST}description = \"changed implementation metadata\"\n"),
    );
    assert!(
        check(&root)
            .findings
            .iter()
            .any(|finding| finding.id == "LOCK-023")
    );
    write(&root, "Cargo.toml", MANIFEST);

    write(&root, "src/template.txt", "changed template\n");
    assert!(
        check(&root)
            .findings
            .iter()
            .any(|finding| finding.id == "LOCK-023")
    );
    reset(&root);
}

#[test]
fn included_rust_implementation_changes_invalidate_macro_authority() {
    let root = repository("included-rust");
    write(
        &root,
        "src/lib.rs",
        r#"//! Macro package.
mod helpers { include!("implementation.rs"); }
pub fn run() { let _ = helpers::reviewed!(); }
"#,
    );
    write(
        &root,
        "src/implementation.rs",
        "//! Included implementation.\nmacro_rules! reviewed { () => { 42 }; }\npub(crate) use reviewed;\n",
    );
    lock(&root);
    assert!(
        !check(&root)
            .findings
            .iter()
            .any(|finding| finding.id.starts_with("LOCK-"))
    );

    write(
        &root,
        "src/implementation.rs",
        "//! Changed implementation.\nmacro_rules! reviewed { () => { 43 }; }\npub(crate) use reviewed;\n",
    );
    assert!(
        check(&root)
            .findings
            .iter()
            .any(|finding| finding.id == "LOCK-023")
    );
    reset(&root);
}

fn repository(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "zrail-macro-input-manifest-{name}-{}",
        std::process::id()
    ));
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create fixture");
    write(&root, "Cargo.toml", MANIFEST);
    write(&root, "zrail.toml", CONTRACT);
    root
}

fn lock(root: &Path) {
    build_lock(root, "zrail.toml".as_ref())
        .expect("build macro implementation lock")
        .write(&root.join("zrail.lock"))
        .expect("write macro implementation lock");
}

fn check(root: &Path) -> zrail_core::Report {
    check_repository(root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check macro implementation fixture")
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
name = "helpers::reviewed"
reason = "Reviewed repository macro implementation."
[source.rust.hygiene]
unsafe = "deny"
lint_suppressions = "allow"
"#;

const MANIFEST: &str = "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n";
