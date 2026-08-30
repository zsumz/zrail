//! One invocation must bind every macro origin selected by a Cargo compilation domain.

use std::{fs, path::Path};

use zrail_core::ReportStatus;
use zrail_rust::{build_lock, check_repository};

#[test]
fn external_and_test_local_origins_are_both_authorized_and_content_bound() {
    let root = std::env::temp_dir().join(format!(
        "zrail-macro-domain-authority-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create fixture");
    write(&root, "Cargo.toml", MANIFEST);
    write(&root, "Cargo.lock", CARGO_LOCK);
    write(&root, "zrail.toml", CONTRACT);
    write(&root, "src/lib.rs", SOURCE);
    build_lock(&root, "zrail.toml".as_ref())
        .expect("build domain authority lock")
        .write(&root.join("zrail.lock"))
        .expect("write domain authority lock");

    let report = check(&root);
    assert_eq!(report.status, ReportStatus::Pass, "{}", report.human());

    write(&root, "src/lib.rs", &SOURCE.replace("=> { 1 }", "=> { 2 }"));
    let report = check(&root);
    assert!(
        report
            .findings
            .iter()
            .any(|finding| { finding.id == "LOCK-023" && finding.message.contains("fixture") }),
        "{}",
        report.human()
    );
    reset(&root);
}

fn check(root: &Path) -> zrail_core::Report {
    check_repository(root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check domain authority fixture")
        .report
}

fn write(root: &Path, path: &str, contents: &str) {
    fs::write(root.join(path), contents).expect("write fixture file");
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

const CARGO_LOCK: &str = r#"version = 4
[[package]]
name = "fixture"
version = "0.0.0"
dependencies = ["serde_json"]
[[package]]
name = "serde_json"
version = "1.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
"#;

const SOURCE: &str = r#"//! Library.
use reviewed_json::json;
#[cfg(test)]
macro_rules! json { ($($tokens:tt)*) => { 1 }; }
pub fn run() { let _ = json!({"ok": true}); }
"#;

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
name = "serde_json::json"
inputs = "opaque"
reason = "Reviewed registry macro expansion."
[source.rust.macros.allow.source]
kind = "registry"
requirement = "1"
[[source.rust.macros.allow]]
name = "json"
definition = "src/lib.rs"
inputs = "opaque"
reason = "Reviewed test-domain repository macro expansion."
[source.rust.hygiene]
unsafe = "deny"
lint_suppressions = "allow"
"#;
