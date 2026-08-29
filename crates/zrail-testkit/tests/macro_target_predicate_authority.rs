//! Target-predicate mounts preserve exact builtin macro origins.

use std::{fs, path::Path};

use zrail_core::ReportStatus;
use zrail_rust::{build_lock, check_repository};

#[test]
fn target_guarded_module_does_not_make_builtin_origin_unresolved() {
    let root = std::env::temp_dir().join(format!(
        "zrail-macro-target-guard-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create fixture");
    write(
        &root,
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
    );
    write(
        &root,
        "src/lib.rs",
        "//! Library.\n#[cfg(unix)]\nmod platform;\nmod owner;\npub struct State { pub epoch: usize }\n",
    );
    write(
        &root,
        "src/platform.rs",
        "//! Platform.\npub fn check(value: bool) { assert!(value); }\n",
    );
    write(
        &root,
        "src/owner.rs",
        "//! Mutation owner.\nuse crate::State;\npub fn advance(state: &mut State) { state.epoch += 1; }\n",
    );
    write(&root, "zrail.toml", CONTRACT);
    build_lock(&root, "zrail.toml".as_ref())
        .expect("build target-guarded lock")
        .write(&root.join("zrail.lock"))
        .expect("write target-guarded lock");

    let report = check_repository(&root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check target-guarded builtin")
        .report;

    assert_eq!(report.status, ReportStatus::Pass, "{}", report.human());
    reset(&root);
}

#[test]
fn target_guarded_local_shadow_cannot_borrow_builtin_authority() {
    let root = std::env::temp_dir().join(format!(
        "zrail-macro-target-shadow-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create fixture");
    write(
        &root,
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
    );
    write(
        &root,
        "src/lib.rs",
        "//! Library.\n#[cfg(unix)]\nmod platform;\n",
    );
    write(
        &root,
        "src/platform.rs",
        "//! Platform.\nmacro_rules! assert { () => {}; }\npub fn check() { assert!(); }\n",
    );
    write(&root, "zrail.toml", CONTRACT);

    let report = check_repository(&root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check target-guarded shadow")
        .report;

    assert!(
        report.findings.iter().any(|finding| {
            finding.id == "RUST-MACRO-006" && finding.path.as_deref() == Some("src/platform.rs")
        }),
        "{}",
        report.human()
    );
    reset(&root);
}

fn write(root: &Path, relative: &str, contents: &str) {
    fs::write(root.join(relative), contents).expect("write fixture");
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
name = "assert"
resolution = "conservative"
field_mutation = "none"
reason = "Reviewed compiler builtin."
[source.rust.hygiene]
unsafe = "deny"
lint_suppressions = "allow"
[[owner]]
name = "state-write"
kind = "field-write"
within = ["src/**"]
match = "crate::State::epoch"
allow = ["src/owner.rs"]
reason = "State writes stay in the owner module."
"#;
