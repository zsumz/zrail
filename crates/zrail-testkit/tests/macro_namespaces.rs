//! Macro lookup is independent from type, module, and value declarations.

use std::{fs, path::Path};

use zrail_core::ReportStatus;
use zrail_rust::{build_lock, check_repository};

#[test]
fn same_spelling_in_other_namespaces_does_not_shadow_a_compiler_macro() {
    let root = std::env::temp_dir().join(format!(
        "zrail-macro-namespace-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create fixture");
    write(&root, "Cargo.toml", MANIFEST);
    write(
        &root,
        "src/lib.rs",
        r#"//! Library.
mod module_case {
    mod format_args {}
    pub fn run() { format_args!("message"); }
}
mod function_case {
    fn format_args() {}
    pub fn run() { format_args!("message"); }
}
mod type_case {
    struct format_args;
    pub fn run() { format_args!("message"); }
}
mod value_case {
    const format_args: () = ();
    pub fn run() { format_args!("message"); }
}
"#,
    );
    write(&root, "zrail.toml", CONTRACT);
    build_lock(&root, "zrail.toml".as_ref())
        .expect("build fixture lock")
        .write(&root.join("zrail.lock"))
        .expect("write fixture lock");

    let report = check_repository(&root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check macro namespace")
        .report;

    assert_eq!(report.status, ReportStatus::Pass, "{}", report.human());
    reset(&root);
}

#[test]
fn restricted_macro_export_does_not_escape_its_logical_module() {
    let root = std::env::temp_dir().join(format!(
        "zrail-macro-visibility-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create fixture");
    write(&root, "Cargo.toml", MANIFEST);
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod outer;\nuse outer::inner::*;\npub fn run() { assert!(true); }\n",
    );
    write(
        &root,
        "src/outer.rs",
        r"//! Outer module.
pub(crate) mod inner {
    macro_rules! assert { ($value:expr) => { $value }; }
    pub(super) use assert;
}
",
    );
    write(
        &root,
        "zrail.toml",
        &CONTRACT.replace("format_args", "assert"),
    );
    build_lock(&root, "zrail.toml".as_ref())
        .expect("build fixture lock")
        .write(&root.join("zrail.lock"))
        .expect("write fixture lock");

    let report = check_repository(&root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check restricted macro visibility")
        .report;

    assert_eq!(report.status, ReportStatus::Pass, "{}", report.human());
    reset(&root);
}

#[test]
fn relative_export_module_wins_over_the_same_crate_root_name() {
    let root = std::env::temp_dir().join(format!(
        "zrail-relative-macro-module-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create fixture");
    write(&root, "Cargo.toml", MANIFEST);
    write(
        &root,
        "src/lib.rs",
        r"//! Library.
mod shared {}
mod bridge {
    mod shared {}
    pub(crate) use shared::*;
}
use bridge::*;
pub fn run() { assert!(true); }
",
    );
    write(
        &root,
        "zrail.toml",
        &CONTRACT.replace("format_args", "assert"),
    );
    build_lock(&root, "zrail.toml".as_ref())
        .expect("build fixture lock")
        .write(&root.join("zrail.lock"))
        .expect("write fixture lock");

    let report = check_repository(&root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check relative macro module")
        .report;

    assert_eq!(report.status, ReportStatus::Pass, "{}", report.human());
    reset(&root);
}

#[test]
fn inaccessible_unknown_glob_does_not_escape_its_module() {
    let root = std::env::temp_dir().join(format!(
        "zrail-private-unknown-macro-export-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create fixture");
    write(&root, "Cargo.toml", MANIFEST);
    write(
        &root,
        "src/lib.rs",
        r"//! Library.
mod hidden {
    use std::*;
}
use hidden::*;
pub fn run() { assert!(true); }
",
    );
    write(
        &root,
        "zrail.toml",
        &CONTRACT.replace("format_args", "assert"),
    );
    build_lock(&root, "zrail.toml".as_ref())
        .expect("build fixture lock")
        .write(&root.join("zrail.lock"))
        .expect("write fixture lock");

    let report = check_repository(&root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check inaccessible unknown glob")
        .report;

    assert_eq!(report.status, ReportStatus::Pass, "{}", report.human());
    reset(&root);
}

#[test]
fn accessible_unanalyzed_glob_reports_an_unknown_export_set() {
    let root = std::env::temp_dir().join(format!(
        "zrail-unknown-macro-export-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create fixture");
    write(&root, "Cargo.toml", MANIFEST);
    write(
        &root,
        "src/lib.rs",
        r"//! Library.
mod bridge {
    pub(crate) use std::*;
}
use bridge::*;
pub fn run() { assert!(true); }
",
    );
    write(
        &root,
        "zrail.toml",
        &CONTRACT.replace("format_args", "assert"),
    );
    build_lock(&root, "zrail.toml".as_ref())
        .expect("build fixture lock")
        .write(&root.join("zrail.lock"))
        .expect("write fixture lock");

    let report = check_repository(&root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check unknown export set")
        .report;
    let rendered = report.human();

    assert_eq!(report.status, ReportStatus::Fail, "{rendered}");
    assert!(rendered.contains("unknown export set"), "{rendered}");
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
    name = "format_args"
reason = "Reviewed compiler expansion."
[source.rust.hygiene]
unsafe = "deny"
lint_suppressions = "allow"
"#;
