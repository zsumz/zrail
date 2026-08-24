//! Exact item-macro manifests produce complete content-bound namespace authority.

use std::{fs, path::PathBuf};

use zrail_core::ReportStatus;
use zrail_rust::{build_lock, check_repository};

#[test]
fn exact_manifest_binds_generated_names_invocation_and_lock_state() {
    let root = fixture();
    let lock = build_lock(&root, "zrail.toml".as_ref()).expect("build exact manifest lock");
    assert_eq!(lock.item_macro_manifests.len(), 1);
    assert_eq!(lock.item_macro_manifests[0].bindings, 1);
    lock.write(&root.join("zrail.lock")).expect("write lock");

    let checked = check_repository(&root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check exact manifested namespace");
    assert_eq!(checked.report.status, ReportStatus::Pass);
    assert!(checked.analysis.is_complete());

    fs::write(root.join("src/lib.rs"), source("unexpected")).expect("change invocation");
    let error = check_repository(&root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect_err("changed invocation tokens must fail before lock construction");
    assert!(error.to_string().contains("invocation tokens differ"));
    reset(&root);
}

fn fixture() -> PathBuf {
    let root =
        std::env::temp_dir().join(format!("zrail-item-macro-manifest-{}", std::process::id()));
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create source");
    fs::create_dir_all(root.join("zrail/macros")).expect("create manifest directory");
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname='fixture'\nversion='0.0.0'\nedition='2024'\n",
    )
    .expect("write Cargo manifest");
    fs::write(root.join("src/lib.rs"), source("")).expect("write source");
    fs::write(root.join("zrail.toml"), contract()).expect("write contract");
    fs::write(root.join("zrail/macros/declare.toml"), manifest()).expect("write exact manifest");
    root
}

fn source(input: &str) -> String {
    format!(
        "//! Exact manifested namespace.\n\
         macro_rules! declare {{ ($($token:tt)*) => {{ struct Generated {{ value: usize }} }} }}\n\
         declare!({input});\n\
         fn accepts_generated(_: Generated) {{}}\n"
    )
}

fn manifest() -> &'static str {
    r#"schema = 1
macro_name = "declare"
invocation_sha256 = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"

[[binding]]
name = "Generated"
kind = "type"
public = false
"#
}

fn contract() -> &'static str {
    r#"schema = 2
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
module_docs = "allow"
facades = "allow"
tests = "allow"

[[source.rust.item_macros]]
name = "declare"
path = "src/lib.rs"
resolution = "exact"
manifest = "zrail/macros/declare.toml"
reason = "The checked-in manifest declares the complete generated namespace."

[source.rust.hygiene]
unsafe = "allow"
lint_suppressions = "allow"
"#
}

fn reset(root: &std::path::Path) {
    if root.exists() {
        fs::remove_dir_all(root).expect("reset fixture");
    }
}
