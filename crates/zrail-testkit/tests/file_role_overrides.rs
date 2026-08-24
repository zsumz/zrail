//! Exact file-role overrides drive facade shape, size, staleness, and explanation.

use std::{fs, path::PathBuf};

use zrail_core::Report;
use zrail_rust::{check_repository, explain_path};

#[test]
fn reasoned_role_overrides_are_effective_and_stale_policy_fails() {
    let root = repository();
    let report = check(&root);

    assert!(has(&report, "RUST-FACADE-001", "src/api.rs"));
    assert!(!has(&report, "RUST-FACADE-001", "src/lib.rs"));
    assert!(has(&report, "RUST-ROLE-001", "src/missing.rs"));
    assert!(has(&report, "RUST-ROLE-003", "src/plain.rs"));
    assert!(!has(&report, "RUST-ROLE-002", "src/main.rs"));
    assert!(!has(&report, "RUST-FACADE-001", "src/main.rs"));

    let api = explain_path(&root, "zrail.toml".as_ref(), "src/api.rs".as_ref())
        .expect("explain facade override");
    assert_eq!(api.inferred_file_role, "implementation");
    assert_eq!(api.effective_file_role, "facade");
    assert_eq!(
        api.file_role_reason.as_deref(),
        Some("Reviewed public surface.")
    );
    assert_eq!(api.design_target, Some(4));
    assert!(api.human().contains("effective role: facade"));

    let library = explain_path(&root, "zrail.toml".as_ref(), "src/lib.rs".as_ref())
        .expect("explain implementation override");
    assert_eq!(library.inferred_file_role, "facade");
    assert_eq!(library.effective_file_role, "implementation");
    assert_eq!(library.design_target, Some(20));

    let entrypoint = explain_path(&root, "zrail.toml".as_ref(), "src/main.rs".as_ref())
        .expect("explain implementation entrypoint override");
    assert_eq!(entrypoint.file_class, "entrypoint");
    assert_eq!(entrypoint.inferred_file_role, "entrypoint");
    assert_eq!(entrypoint.effective_file_role, "implementation");
    assert_eq!(
        entrypoint.file_role_reason.as_deref(),
        Some("Reviewed single-file binary.")
    );
    assert_eq!(entrypoint.design_target, Some(20));
    assert_eq!(entrypoint.declarative_shape, None);
    reset(&root);
}

#[test]
fn entrypoints_accept_only_an_implementation_override() {
    let root = repository();
    let contract = fs::read_to_string(root.join("zrail.toml")).expect("read contract");
    let contract = contract.replace(
        concat!(
            "path = \"src/main.rs\"\n",
            "role = \"implementation\"\n",
            "reason = \"Reviewed single-file binary.\"",
        ),
        concat!(
            "path = \"src/main.rs\"\n",
            "role = \"facade\"\n",
            "reason = \"Invalid entrypoint role.\"",
        ),
    );
    write(&root, "zrail.toml", &contract);

    let report = check(&root);
    assert!(has(&report, "RUST-ROLE-002", "src/main.rs"));
    assert!(has(&report, "RUST-FACADE-001", "src/main.rs"));
    reset(&root);
}

fn repository() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "zrail-file-role-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create source directory");
    write(&root, "Cargo.toml", MANIFEST);
    write(&root, "zrail.toml", CONTRACT);
    write(&root, "src/lib.rs", LIBRARY);
    write(
        &root,
        "src/main.rs",
        "//! Binary entrypoint.\nfn helper() -> u8 { 1 }\nfn main() { let _ = helper(); }\n",
    );
    write(
        &root,
        "src/api.rs",
        "//! API.\npub fn value() -> u8 { 1 }\n",
    );
    write(
        &root,
        "src/plain.rs",
        "//! Plain implementation.\npub const VALUE: u8 = 1;\n",
    );
    root
}

fn check(root: &std::path::Path) -> Report {
    check_repository(root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check file-role fixture")
        .report
}

fn has(report: &Report, id: &str, path: &str) -> bool {
    report
        .findings
        .iter()
        .any(|finding| finding.id == id && finding.path.as_deref() == Some(path))
}

fn write(root: &std::path::Path, path: &str, contents: &str) {
    fs::write(root.join(path), contents).expect("write fixture");
}

fn reset(root: &PathBuf) {
    if root.exists() {
        fs::remove_dir_all(root).expect("reset fixture");
    }
}

const MANIFEST: &str = r#"[package]
name = "fixture"
version = "0.0.0"
edition = "2024"
"#;

const LIBRARY: &str = concat!(
    "//! Single-file implementation root.\n",
    "mod api;\n",
    "mod plain;\n",
    "pub fn run() -> u8 { api::value() + plain::VALUE }\n",
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
module_docs = "required"
facades = "declarative"
tests = "allow"
entrypoints = "declarative"
[source.rust.macros]
mode = "allow"
[source.rust.hygiene]
unsafe = "deny"
lint_suppressions = "allow"
[source.rust.size.facade]
target = 4
hard = 10
[source.rust.size.implementation]
target = 20
hard = 30
[source.rust.size.test]
target = 20
hard = 30
[source.rust.size.auxiliary]
target = 20
hard = 30

[[source.rust.file_roles]]
path = "src/api.rs"
role = "facade"
reason = "Reviewed public surface."

[[source.rust.file_roles]]
path = "src/lib.rs"
role = "implementation"
reason = "Intentional single-file crate."

[[source.rust.file_roles]]
path = "src/main.rs"
role = "implementation"
reason = "Reviewed single-file binary."

[[source.rust.file_roles]]
path = "src/plain.rs"
role = "implementation"
reason = "Redundant policy must fail."

[[source.rust.file_roles]]
path = "src/missing.rs"
role = "facade"
reason = "Missing policy must fail."
"#;
