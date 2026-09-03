//! Qualified aliases crossing include splices retain repository macro authority.

use std::{fs, path::Path};

use zrail_core::{Report, ReportStatus};
use zrail_rust::{build_lock, check_repository};

#[test]
fn qualified_include_alias_resolves_exact_repository_authority() {
    let root = fixture("exact", REPOSITORY_ALLOWANCE);
    write_source(&root);
    write_lock(&root);

    let report = check(&root);

    assert_eq!(report.status, ReportStatus::Pass, "{}", report.human());
    reset(&root);
}

fn write_source(root: &Path) {
    write(
        root,
        "src/lib.rs",
        r#"//! Library.
mod local {
    macro_rules! reviewed { ($($tokens:tt)*) => { 1 }; }
    pub(crate) use reviewed;
}
include!("imports.rs");
pub fn run() { reviewed_namespace::reviewed!(); }
"#,
    );
    write(
        root,
        "src/imports.rs",
        "use crate::local as reviewed_namespace;\n",
    );
}

fn fixture(name: &str, allowances: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!(
        "zrail-qualified-include-alias-{name}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create fixture");
    write(&root, "Cargo.toml", MANIFEST);
    write(&root, "zrail.toml", &format!("{CONTRACT}{allowances}"));
    root
}

fn check(root: &Path) -> Report {
    check_repository(root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check qualified include alias fixture")
        .report
}

fn write_lock(root: &Path) {
    build_lock(root, "zrail.toml".as_ref())
        .expect("build qualified include alias lock")
        .write(&root.join("zrail.lock"))
        .expect("write qualified include alias lock");
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
module_docs = "allow"
facades = "allow"
tests = "allow"
[source.rust.macros]
mode = "deny-unreviewed"
[source.rust.hygiene]
unsafe = "deny"
lint_suppressions = "allow"
"#;

const REPOSITORY_ALLOWANCE: &str = r#"
[[source.rust.macros.allow]]
name = "crate::local::reviewed"
definition = "src/lib.rs"
reason = "Reviewed repository macro expansion."
"#;
