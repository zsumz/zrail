//! Import lookup keeps Rust's macro namespace separate from other symbols.

use std::{fs, path::Path};

use zrail_core::ReportStatus;
use zrail_rust::{build_lock, check_repository};

#[test]
fn non_macro_imports_do_not_shadow_a_compiler_macro() {
    let root = fixture("non-macro-imports");
    write(
        &root,
        "src/lib.rs",
        r#"//! Library.
mod module_symbols { pub mod format_args {} }
mod function_symbols { pub fn format_args() {} }
mod type_symbols { pub struct format_args; }
mod constant_symbols { pub const format_args: () = (); }
mod module_case {
    use crate::module_symbols::format_args;
    pub fn run() { format_args!("message"); }
}
mod function_case {
    use crate::function_symbols::format_args;
    pub fn run() { format_args!("message"); }
}
mod type_case {
    use crate::type_symbols::format_args;
    pub fn run() { format_args!("message"); }
}
mod constant_case {
    use crate::constant_symbols::format_args;
    pub fn run() { format_args!("message"); }
}
"#,
    );
    write(
        &root,
        "zrail.toml",
        &format!("{CONTRACT}{COMPILER_ALLOWANCE}{HYGIENE}"),
    );

    let report = check(&root);

    assert_eq!(report.status, ReportStatus::Pass, "{}", report.human());
    reset(&root);
}

#[test]
fn one_use_can_bind_macro_and_value_namespaces_independently() {
    let root = fixture("shared-import");
    write(
        &root,
        "src/lib.rs",
        r"//! Library.
mod symbols {
    macro_rules! reviewed { () => {}; }
    pub(crate) use reviewed;
    pub fn reviewed() {}
}
use symbols::reviewed;
pub fn run() { reviewed!(); }
",
    );
    write(
        &root,
        "zrail.toml",
        &format!("{CONTRACT}{REPOSITORY_ALLOWANCE}{HYGIENE}"),
    );

    let report = check(&root);

    assert_eq!(report.status, ReportStatus::Pass, "{}", report.human());
    reset(&root);
}

fn fixture(name: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!(
        "zrail-macro-namespace-{name}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create fixture");
    write(&root, "Cargo.toml", MANIFEST);
    root
}

fn check(root: &Path) -> zrail_core::Report {
    build_lock(root, "zrail.toml".as_ref())
        .expect("build namespace lock")
        .write(&root.join("zrail.lock"))
        .expect("write namespace lock");
    check_repository(root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check namespace fixture")
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
"#;

const COMPILER_ALLOWANCE: &str = r#"[[source.rust.macros.allow]]
name = "format_args"
reason = "Reviewed compiler expansion."
"#;

const REPOSITORY_ALLOWANCE: &str = r#"[[source.rust.macros.allow]]
name = "symbols::reviewed"
definition = "src/lib.rs"
reason = "Reviewed repository expansion."
"#;

const HYGIENE: &str = r#"[source.rust.hygiene]
unsafe = "deny"
lint_suppressions = "allow"
"#;
