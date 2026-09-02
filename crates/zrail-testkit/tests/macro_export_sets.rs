//! Repository globs contribute only macros exported by their target namespace.

use std::{fs, path::Path};

use zrail_core::ReportStatus;
use zrail_rust::{build_lock, check_repository};

#[test]
fn nested_empty_export_globs_do_not_shadow_compiler_macros() {
    let root = std::env::temp_dir().join(format!(
        "zrail-empty-macro-exports-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create fixture");
    write(&root, "Cargo.toml", MANIFEST);
    write(
        &root,
        "src/lib.rs",
        concat!(
            "//! Library.\n",
            "mod hidden;\n",
            "mod exports;\n",
            "mod prelude;\n",
            "use prelude::*;\n",
            "pub fn run() {\n",
            "    assert!(true);\n",
            "    assert_eq!(1, 1);\n",
            "    let _ = format!(\"message\");\n",
            "}\n",
        ),
    );
    write(
        &root,
        "src/hidden.rs",
        r"//! Private macros.
macro_rules! assert { ($($token:tt)*) => {}; }
macro_rules! assert_eq { ($($token:tt)*) => {}; }
macro_rules! format { ($($token:tt)*) => { String::new() }; }
",
    );
    write(
        &root,
        "src/exports.rs",
        "//! Empty public macro surface.\npub use crate::hidden::*;\n",
    );
    write(
        &root,
        "src/prelude.rs",
        "//! Chained empty macro surface.\npub use crate::exports::*;\n",
    );
    write(
        &root,
        "zrail.toml",
        &format!("{CONTRACT_PREFIX}{COMPILER_ALLOWANCES}{CONTRACT_SUFFIX}"),
    );
    build_lock(&root, "zrail.toml".as_ref())
        .expect("build fixture lock")
        .write(&root.join("zrail.lock"))
        .expect("write fixture lock");

    let report = check_repository(&root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check empty macro exports")
        .report;

    assert_eq!(report.status, ReportStatus::Pass, "{}", report.human());
    reset(&root);
}

#[test]
fn one_repository_macro_resolves_through_nested_export_globs() {
    let root = std::env::temp_dir().join(format!(
        "zrail-nested-macro-export-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create fixture");
    write(&root, "Cargo.toml", MANIFEST);
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod hidden;\nmod exports;\nmod prelude;\nuse prelude::*;\npub fn run() { reviewed!(); }\n",
    );
    write(
        &root,
        "src/hidden.rs",
        "//! Macro definition.\nmacro_rules! reviewed { () => {}; }\npub(crate) use reviewed;\n",
    );
    write(
        &root,
        "src/exports.rs",
        "//! First export layer.\npub(crate) use crate::hidden::*;\n",
    );
    write(
        &root,
        "src/prelude.rs",
        "//! Second export layer.\npub(crate) use crate::exports::*;\n",
    );
    write(
        &root,
        "zrail.toml",
        &format!(
            r#"{CONTRACT_PREFIX}[[source.rust.macros.allow]]
name = "prelude::reviewed"
definition = "src/hidden.rs"
reason = "Reviewed repository expansion."
{CONTRACT_SUFFIX}"#,
        ),
    );
    build_lock(&root, "zrail.toml".as_ref())
        .expect("build fixture lock")
        .write(&root.join("zrail.lock"))
        .expect("write fixture lock");

    let report = check_repository(&root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check nested macro export")
        .report;

    assert_eq!(report.status, ReportStatus::Pass, "{}", report.human());
    reset(&root);
}

#[test]
fn repository_dependency_macro_resolves_through_a_reexport_glob() {
    let root = std::env::temp_dir().join(format!(
        "zrail-dependency-macro-export-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    for package in ["macro_impl", "bridge", "consumer"] {
        fs::create_dir_all(root.join(format!("crates/{package}/src")))
            .expect("create package fixture");
    }
    write(&root, "Cargo.toml", WORKSPACE_MANIFEST);
    write(
        &root,
        "crates/macro_impl/Cargo.toml",
        &package_manifest("macro-impl", ""),
    );
    write(
        &root,
        "crates/macro_impl/src/lib.rs",
        "//! Macro implementation.\n#[macro_export]\nmacro_rules! reviewed { () => {}; }\n",
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
        "crates/consumer/Cargo.toml",
        &package_manifest("consumer", "bridge = { path = \"../bridge\" }\n"),
    );
    write(
        &root,
        "crates/consumer/src/lib.rs",
        "//! Macro consumer.\nuse bridge::*;\npub fn run() { reviewed!(); }\n",
    );
    write(&root, "zrail.toml", WORKSPACE_CONTRACT);
    build_lock(&root, "zrail.toml".as_ref())
        .expect("build fixture lock")
        .write(&root.join("zrail.lock"))
        .expect("write fixture lock");

    let report = check_repository(&root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check dependency macro export")
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

fn package_manifest(name: &str, dependencies: &str) -> String {
    format!(
        "[package]\nname = \"{name}\"\nversion = \"0.0.0\"\nedition = \"2024\"\n[dependencies]\n{dependencies}"
    )
}

const MANIFEST: &str = "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n";

const WORKSPACE_MANIFEST: &str = "[workspace]\nmembers = [\"crates/*\"]\nresolver = \"3\"\n";

const CONTRACT_PREFIX: &str = r#"schema = 1
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
"#;

const COMPILER_ALLOWANCES: &str = r#"
[[source.rust.macros.allow]]
name = "assert"
reason = "Reviewed compiler expansion."
[[source.rust.macros.allow]]
name = "assert_eq"
reason = "Reviewed compiler expansion."
[[source.rust.macros.allow]]
name = "format"
reason = "Reviewed compiler expansion."
"#;

const CONTRACT_SUFFIX: &str = r#"
[source.rust.hygiene]
unsafe = "deny"
lint_suppressions = "allow"
"#;

const WORKSPACE_CONTRACT: &str = r#"schema = 1
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
definition = "crates/macro_impl/src/lib.rs"
reason = "Reviewed repository dependency expansion."
[source.rust.hygiene]
unsafe = "deny"
lint_suppressions = "allow"
"#;
