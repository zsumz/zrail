//! Integration tests resolve the package's own library as an extern-prelude root.

use std::{fs, path::Path};

use zrail_core::ReportStatus;
use zrail_rust::{build_lock, check_repository};

#[test]
fn integration_test_macro_import_binds_the_crate_under_test() {
    let root = std::env::temp_dir().join(format!(
        "zrail-integration-crate-root-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create source fixture");
    fs::create_dir_all(root.join("tests")).expect("create integration fixture");
    write(&root, "Cargo.toml", MANIFEST);
    write(
        &root,
        "src/lib.rs",
        "//! Library.\n#[macro_export]\nmacro_rules! reviewed { () => {}; }\n",
    );
    write(
        &root,
        "tests/api.rs",
        "//! Public API checks.\nuse fixture::reviewed;\n#[test]\nfn api() { reviewed!(); }\n",
    );
    write(&root, "zrail.toml", CONTRACT);
    build_lock(&root, "zrail.toml".as_ref())
        .expect("build fixture lock")
        .write(&root.join("zrail.lock"))
        .expect("write fixture lock");

    let report = check_repository(&root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check crate-under-test macro")
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
name = "fixture::reviewed"
reason = "Reviewed crate-under-test macro."
[source.rust.macros.allow.source]
kind = "repository"
package = "fixture"
directory = "."
ambient_inputs = "none"
[source.rust.hygiene]
unsafe = "deny"
lint_suppressions = "allow"
"#;
