//! Effective Rust crate roots are inspected, attested, locked, and policy-visible.

use std::{fs, path::PathBuf};

use zrail_core::{ReportStatus, compare_architecture, load_contract};
use zrail_rust::{build_lock, check_repository};

#[path = "cargo_crate_root/contracts.rs"]
mod contracts;

use contracts::{BASE_CONTRACT, PACKAGE, UNCONSTRAINED_CONTRACT, WORKSPACE};

#[test]
fn custom_workspace_library_name_is_locked_and_canonicalized() {
    let root = workspace("custom-lib", "tokio = { path = \"../tokio\" }", "runtime");
    write(
        &root,
        "crates/app/src/lib.rs",
        "//! App.\npub fn run() { runtime::process::Command::new(\"sh\"); }\n",
    );

    let result = check(&root, BASE_CONTRACT);
    assert_effect(&result.report, "runtime::process");
    let dependency = dependency(&result.candidate_lock, "app", "tokio");
    assert_eq!(dependency.crate_root.as_deref(), Some("runtime"));
    reset(&root);
}

#[test]
fn explicit_package_rename_is_the_source_visible_root() {
    let root = workspace(
        "explicit-rename",
        "async-runtime = { package = \"tokio\", path = \"../tokio\" }",
        "runtime",
    );
    write(
        &root,
        "crates/app/src/lib.rs",
        "//! App.\npub fn run() { async_runtime::process::Command::new(\"sh\"); }\n",
    );

    let result = check(&root, BASE_CONTRACT);
    assert_effect(&result.report, "async_runtime::process");
    let dependency = dependency(&result.candidate_lock, "app", "async-runtime");
    assert_eq!(dependency.name, "tokio");
    assert_eq!(dependency.crate_root.as_deref(), Some("async_runtime"));
    reset(&root);
}

#[test]
fn external_crate_root_requires_explicit_authority() {
    let root = standalone("unresolved", "tokio = \"1\"");

    let result = check(&root, BASE_CONTRACT);

    assert!(result.report.findings.iter().any(|finding| {
        finding.id == "CARGO-IDENTITY-001" && finding.message.contains("tokio")
    }));
    reset(&root);
}

#[test]
fn irrelevant_external_crate_root_remains_explicitly_unresolved() {
    let root = standalone("unresolved-irrelevant", "serde = \"1\"");

    let result = analyze(&root, UNCONSTRAINED_CONTRACT);
    let dependency = dependency(&result.candidate_lock, "app", "serde");

    assert_eq!(result.report.status, ReportStatus::Pass);
    assert_eq!(dependency.crate_root, None);
    reset(&root);
}

#[test]
fn canonical_macro_allowance_requires_exact_external_identity() {
    let root = standalone("unresolved-macro", "tokio = \"1\"");
    write(
        &root,
        "src/lib.rs",
        "//! App.\npub fn run() { tokio::select! {} }\n",
    );
    let contract = UNCONSTRAINED_CONTRACT.replace(
        "[source.rust.hygiene]",
        "[source.rust.macros]\nmode = \"deny-unreviewed\"\n\n[[source.rust.macros.allow]]\nname = \"tokio::select\"\nreason = \"Reviewed expansion.\"\n\n[source.rust.hygiene]",
    );

    let result = check(&root, &contract);

    assert!(
        result
            .report
            .findings
            .iter()
            .any(|finding| finding.id == "CARGO-IDENTITY-001")
    );
    reset(&root);
}

#[test]
fn attested_external_crate_root_is_policy_visible_and_must_remain_used() {
    let root = standalone("attested", "tokio = \"1\"");
    write(
        &root,
        "src/lib.rs",
        "//! App.\npub fn run() { runtime::process::Command::new(\"sh\"); }\n",
    );
    let contract = format!(
        "{BASE_CONTRACT}\n[[dependencies.crate_root]]\npackage = \"tokio\"\nroot = \"runtime\"\nreason = \"Reviewed crate metadata establishes the external Rust root.\"\n"
    );

    let result = check(&root, &contract);
    assert_effect(&result.report, "runtime::process");
    assert_eq!(
        dependency(&result.candidate_lock, "app", "tokio")
            .crate_root
            .as_deref(),
        Some("runtime")
    );
    write(&root, "Cargo.toml", PACKAGE);
    let stale = check(&root, &contract);
    assert!(
        stale
            .report
            .findings
            .iter()
            .any(|finding| finding.id == "CARGO-IDENTITY-002")
    );
    reset(&root);
}

#[test]
fn custom_library_name_change_changes_locked_architecture() {
    let root = workspace("lock-change", "tokio = { path = \"../tokio\" }", "runtime");
    write(&root, "zrail.toml", BASE_CONTRACT);
    let before = build_lock(&root, "zrail.toml".as_ref()).expect("build before lock");
    let manifest = root.join("crates/tokio/Cargo.toml");
    let changed = fs::read_to_string(&manifest)
        .expect("read dependency manifest")
        .replace("name = \"runtime\"", "name = \"executor\"");
    fs::write(manifest, changed).expect("change library name");
    let after = build_lock(&root, "zrail.toml".as_ref()).expect("build after lock");
    let contract = load_contract(&root, "zrail.toml".as_ref()).expect("load contract");
    let diff = compare_architecture(
        &contract.contract,
        Some(&before),
        &contract.contract,
        Some(&after),
    );

    assert_ne!(
        dependency(&before, "app", "tokio").crate_root,
        dependency(&after, "app", "tokio").crate_root
    );
    assert!(diff.denies_grants(), "{}", diff.human());
    for root in ["runtime", "executor"] {
        assert!(
            diff.changes
                .iter()
                .any(|change| change.subject.contains(root))
        );
    }
    reset(&root);
}

fn assert_effect(report: &zrail_core::Report, observed: &str) {
    assert!(
        report
            .findings
            .iter()
            .any(|finding| { finding.id == "EFFECT-001" && finding.message.contains(observed) }),
        "missing canonical effect for {observed}: {:#?}",
        report.findings
    );
}

fn dependency<'a>(
    lock: &'a zrail_core::LockFile,
    package: &str,
    alias: &str,
) -> &'a zrail_core::LockedDependency {
    lock.packages
        .iter()
        .find(|candidate| candidate.name == package)
        .and_then(|candidate| {
            candidate
                .dependencies
                .iter()
                .find(|dependency| dependency.alias.as_deref() == Some(alias))
        })
        .unwrap_or_else(|| panic!("missing {package}:{alias}"))
}

fn check(root: &std::path::Path, contract: &str) -> zrail_rust::CheckResult {
    let result = analyze(root, contract);
    assert_ne!(result.report.status, ReportStatus::Pass);
    result
}

fn analyze(root: &std::path::Path, contract: &str) -> zrail_rust::CheckResult {
    fs::write(root.join("zrail.toml"), contract).expect("write contract");
    check_repository(root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("analyze crate-root fixture")
}

fn workspace(name: &str, dependency: &str, library: &str) -> PathBuf {
    let root = root(name);
    for package in ["app", "tokio"] {
        fs::create_dir_all(root.join(format!("crates/{package}/src"))).expect("create package");
    }
    write(&root, "Cargo.toml", WORKSPACE);
    write(
        &root,
        "crates/app/Cargo.toml",
        &format!("{PACKAGE}\n[dependencies]\n{dependency}\n"),
    );
    write(
        &root,
        "crates/tokio/Cargo.toml",
        &format!(
            "[package]\nname = \"tokio\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\n[lib]\nname = \"{library}\"\n"
        ),
    );
    write(&root, "crates/tokio/src/lib.rs", "//! Runtime.\n");
    root
}

fn standalone(name: &str, dependency: &str) -> PathBuf {
    let root = root(name);
    fs::create_dir_all(root.join("src")).expect("create source");
    write(
        &root,
        "Cargo.toml",
        &format!("{PACKAGE}\n[dependencies]\n{dependency}\n"),
    );
    write(&root, "src/lib.rs", "//! App.\npub fn run() {}\n");
    root
}

fn root(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "zrail-crate-root-{name}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    root
}

fn write(root: &std::path::Path, path: &str, contents: &str) {
    fs::write(root.join(path), contents).expect("write fixture file");
}

fn reset(root: &std::path::Path) {
    if root.exists() {
        fs::remove_dir_all(root).expect("reset fixture");
    }
}
