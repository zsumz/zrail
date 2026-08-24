//! External crate-root authority is bound to one exact direct dependency source.

use std::{fs, path::PathBuf};

use zrail_rust::check_repository;

#[test]
fn same_name_registry_and_git_dependencies_use_distinct_attestations() {
    let root = repository();
    let first = check(&root);
    let candidate = first.candidate_lock.as_ref().expect("complete candidate");
    let registry = dependency(candidate, "registry-app");
    let git = dependency(candidate, "git-app");
    assert_eq!(registry.crate_root.as_deref(), Some("registry_runtime"));
    assert_eq!(git.crate_root.as_deref(), Some("git_runtime"));
    assert_eq!(
        first
            .report
            .findings
            .iter()
            .filter(|finding| finding.id == "CAP-001")
            .count(),
        2
    );
    assert!(!first.report.findings.iter().any(|finding| {
        matches!(
            finding.id.as_str(),
            "CARGO-IDENTITY-001" | "CARGO-IDENTITY-002"
        )
    }));
    let macro_findings = first
        .report
        .findings
        .iter()
        .filter(|finding| finding.id == "RUST-MACRO-006")
        .collect::<Vec<_>>();
    assert_eq!(macro_findings.len(), 1);
    assert!(
        macro_findings[0]
            .path
            .as_deref()
            .is_some_and(|path| path.contains("git-app"))
    );

    let manifest = root.join("crates/git-app/Cargo.toml");
    let changed = fs::read_to_string(&manifest)
        .expect("read Git consumer")
        .replace("rev = \"abc123\"", "rev = \"def456\"");
    fs::write(manifest, changed).expect("change exact Git source identity");
    let changed = check(&root);
    for id in ["CARGO-IDENTITY-001", "CARGO-IDENTITY-002"] {
        assert!(
            changed
                .report
                .findings
                .iter()
                .any(|finding| finding.id == id)
        );
    }
    reset(&root);
}

fn dependency<'a>(
    lock: &'a zrail_core::LockFile,
    package: &str,
) -> &'a zrail_core::LockedDependency {
    lock.packages
        .iter()
        .find(|candidate| candidate.name == package)
        .and_then(|candidate| candidate.dependencies.first())
        .unwrap_or_else(|| panic!("missing dependency for {package}"))
}

fn check(root: &std::path::Path) -> zrail_rust::CheckResult {
    check_repository(root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check exact crate-root authority fixture")
}

fn repository() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "zrail-crate-root-authority-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    for package in ["registry-app", "git-app"] {
        fs::create_dir_all(root.join(format!("crates/{package}/src"))).expect("create package");
    }
    write(
        &root,
        "Cargo.toml",
        "[workspace]\nmembers = [\"crates/registry-app\", \"crates/git-app\"]\nresolver = \"3\"\n",
    );
    write(
        &root,
        "crates/registry-app/Cargo.toml",
        &manifest("registry-app", "foo = \"1\""),
    );
    write(
        &root,
        "crates/git-app/Cargo.toml",
        &manifest(
            "git-app",
            "foo = { git = \"https://example.invalid/foo\", rev = \"abc123\" }",
        ),
    );
    write(
        &root,
        "crates/registry-app/src/lib.rs",
        "//! Registry.\npub fn run() { registry_runtime::danger(); registry_runtime::select! {} }\n",
    );
    write(
        &root,
        "crates/git-app/src/lib.rs",
        "//! Git.\npub fn run() { git_runtime::danger(); git_runtime::select! {} }\n",
    );
    write(&root, "zrail.toml", CONTRACT);
    root
}

fn manifest(name: &str, dependency: &str) -> String {
    format!(
        "[package]\nname = \"{name}\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\n[dependencies]\n{dependency}\n"
    )
}

fn write(root: &std::path::Path, path: &str, contents: &str) {
    fs::write(root.join(path), contents).expect("write fixture");
}

fn reset(root: &PathBuf) {
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
[[dependencies.crate_root]]
package = "foo"
root = "registry_runtime"
reason = "Reviewed registry metadata."
[dependencies.crate_root.source]
kind = "registry"
requirement = "1"
[[dependencies.crate_root]]
package = "foo"
root = "git_runtime"
reason = "Reviewed Git metadata."
[dependencies.crate_root.source]
kind = "git"
repository = "https://example.invalid/foo"
rev = "abc123"
[source.rust]
module_docs = "required"
facades = "allow"
tests = "allow"
[source.rust.macros]
mode = "deny-unreviewed"
[[source.rust.macros.allow]]
name = "foo::select"
reason = "Only the exact registry macro implementation is reviewed."
[source.rust.macros.allow.source]
kind = "registry"
requirement = "1"
[source.rust.hygiene]
unsafe = "deny"
lint_suppressions = "allow"
[[scope]]
name = "deny-danger"
include = ["crates/**/src/**/*.rs"]
reason = "Dependency calls remain canonical."
[scope.symbols]
deny = ["foo::danger"]
"#;
