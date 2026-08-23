//! Qualified aliases crossing include splices cannot borrow dependency authority.

use std::{fs, path::Path};

use zrail_core::Report;
use zrail_rust::{build_lock, check_repository};

#[test]
fn qualified_include_alias_requires_conservative_written_authority() {
    let exact = fixture("exact", EXTERNAL_ONLY);
    write_source(&exact);
    write_lock(&exact);
    assert_macro_failure(&check(&exact));
    reset(&exact);

    let conservative = fixture(
        "conservative",
        &format!("{EXTERNAL_ONLY}{CONSERVATIVE_WRITTEN}"),
    );
    write_source(&conservative);
    write_lock(&conservative);
    assert_no_macro_failure(&check(&conservative));
    reset(&conservative);
}

fn write_source(root: &Path) {
    write(
        root,
        "src/lib.rs",
        r#"//! Library.
mod local {
    macro_rules! json { ($($tokens:tt)*) => { 1 }; }
    pub(crate) use json;
}
include!("imports.rs");
pub fn run() { reviewed_json::json!({"ok": true}); }
"#,
    );
    write(
        root,
        "src/imports.rs",
        "use crate::local as reviewed_json;\n",
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

fn assert_macro_failure(report: &Report) {
    assert!(
        report.findings.iter().any(|finding| {
            finding.id.starts_with("RUST-MACRO-") && finding.message.contains("reviewed_json::json")
        }),
        "{}",
        report.human()
    );
}

fn assert_no_macro_failure(report: &Report) {
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.id.starts_with("RUST-MACRO-")),
        "{}",
        report.human()
    );
}

fn write(root: &Path, path: &str, contents: &str) {
    fs::write(root.join(path), contents).expect("write fixture");
}

fn reset(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).expect("reset fixture");
    }
}

const MANIFEST: &str = concat!(
    "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
    "[dependencies]\nreviewed_json = { package = \"serde_json\", version = \"1\" }\n",
);

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

const EXTERNAL_ONLY: &str = r#"
[[source.rust.macros.allow]]
name = "serde_json::json"
inputs = "opaque"
reason = "Reviewed registry macro expansion."
[source.rust.macros.allow.source]
kind = "registry"
requirement = "1"
"#;

const CONSERVATIVE_WRITTEN: &str = r#"
[[source.rust.macros.allow]]
name = "reviewed_json::json"
binding = "conservative"
inputs = "opaque"
reason = "Reviewed unresolved qualified include alias."
"#;
