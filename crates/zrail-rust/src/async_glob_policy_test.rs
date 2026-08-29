//! Repository checks keep runtime-neutral syntax and written glob hygiene distinct.

use std::{fs, path::PathBuf};

use super::check_repository;

#[test]
fn profiles_deny_async_syntax_and_globs_keep_only_facade_reexports() {
    let root = repository("facade-reexports-only");
    write(
        &root,
        "src/lib.rs",
        "//! facade\nmod worker;\npub mod api { pub fn value() {} }\npub use api::*;\n",
    );
    write(
        &root,
        "src/worker.rs",
        r"//! implementation
use crate::api::*;
macro_rules! sync_value { () => { 1 } }
pub fn macro_value() { let _ = sync_value!(); }
#[cfg(any())]
pub async fn absent() { let _ = sync_value!(); }
pub async fn work() {
    let future = async { 1 };
    let closure = async || 2;
    let _ = future.await;
    let _ = closure;
}
",
    );

    let result = check_repository(&root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check repository");
    let report = result.report.human();

    assert_eq!(error_count(&report, "RUST-HYG-009"), 1, "{report}");
    assert_eq!(error_count(&report, "SYNTAX-001"), 4, "{report}");
    assert_eq!(error_count(&report, "SYNTAX-002"), 1, "{report}");
    assert!(!report.contains("EFFECT-001"), "{report}");
    reset(&root);
}

#[test]
fn test_super_mode_uses_semantic_test_reachability_and_exact_spelling() {
    let root = repository("facade-reexports-and-test-super");
    write(
        &root,
        "src/lib.rs",
        r"//! facade
#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;
    pub use super::*;
}
#[cfg(test)]
mod worker_test /* fixture declaration */;
#[cfg(any(test, unix))]
use super::*;
use super::*;
",
    );
    write(
        &root,
        "src/worker_test.rs",
        "//! test-only module\nuse super::*;\nuse crate::*;\n",
    );

    let result = check_repository(&root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check repository");
    let report = result.report.human();

    assert_eq!(error_count(&report, "RUST-HYG-009"), 5, "{report}");
    reset(&root);
}

fn error_count(report: &str, id: &str) -> usize {
    report
        .lines()
        .filter(|line| line.starts_with(&format!("error[{id}]")))
        .count()
}

fn repository(glob_imports: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "zrail-async-glob-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create source");
    write(
        &root,
        "Cargo.toml",
        "[package]\nname = \"policy-app\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
    );
    write(
        &root,
        "zrail.toml",
        &CONTRACT.replace("$GLOB_IMPORTS", glob_imports),
    );
    root
}

fn write(root: &std::path::Path, path: &str, content: &str) {
    fs::write(root.join(path), content).expect("write fixture");
}

fn reset(root: &std::path::Path) {
    if root.exists() {
        fs::remove_dir_all(root).expect("reset fixture");
    }
}

const CONTRACT: &str = r#"schema = 2
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
cycles = "allow"

[source.rust]
module_docs = "allow"
facades = "allow"
tests = "allow"

[source.rust.hygiene]
unsafe = "allow"
lint_suppressions = "allow"
glob_imports = "$GLOB_IMPORTS"

[profiles.sync.effects]
deny = []

[profiles.sync.syntax]
deny = ["async-fn", "async-block", "async-closure", "await"]

[[layer]]
name = "core"
packages = ["policy-app"]
may_depend_on = []
profiles = ["sync"]
reason = "Core remains runtime-neutral and synchronous."
"#;
