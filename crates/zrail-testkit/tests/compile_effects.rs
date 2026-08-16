//! Compile-time environment and file embedding are distinct, exact effects.

use std::{fs, path::PathBuf};

use zrail_core::Report;
use zrail_rust::check_repository;

#[test]
fn compile_environment_and_filesystem_have_explicit_effects() {
    let root = repository("effects");
    write(&root, "src/data.txt", "fixture\n");
    write(
        &root,
        "src/lib.rs",
        "//! Compile effects.\npub const HOME: &str = concat!(env!(\"HOME\"));\npub const DATA: &str = include_str!(\"data.txt\");\n",
    );

    let report = check(&root);
    for effect in ["CompileEnvironment", "CompileFilesystem"] {
        assert!(
            report
                .findings
                .iter()
                .any(|finding| { finding.id == "EFFECT-001" && finding.message.contains(effect) })
        );
    }
    reset(&root);
}

#[test]
fn embedded_files_must_be_literal_inventoried_repository_input() {
    let root = repository("paths");
    write(
        &root,
        "src/lib.rs",
        "//! Escaping input.\npub const DATA: &str = include_str!(\"../../outside.txt\");\n",
    );

    let report = check(&root);
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.id == "RUST-COMPILE-001")
    );
    reset(&root);
}

#[test]
fn local_macro_shadow_cannot_claim_a_compiler_intrinsic_effect() {
    let root = repository("shadow");
    write(
        &root,
        "src/lib.rs",
        "//! Shadow.\nmacro_rules! env { ($name:literal) => { \"local\" }; }\npub const VALUE: &str = env!(\"HOME\");\n",
    );

    let report = check(&root);
    assert!(!report.findings.iter().any(|finding| {
        finding.id == "EFFECT-001" && finding.message.contains("CompileEnvironment")
    }));
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.id == "RUST-MACRO-001")
    );
    reset(&root);
}

#[test]
fn external_macro_named_env_is_not_a_compiler_intrinsic() {
    let root = repository("external-env");
    write(
        &root,
        "Cargo.toml",
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n[dependencies]\nenv = \"1\"\n",
    );
    write(
        &root,
        "src/lib.rs",
        "//! External env macro.\npub const VALUE: &str = env!(\"HOME\");\n",
    );

    let report = check(&root);
    assert!(
        !report.findings.iter().any(|finding| {
            finding.id == "EFFECT-001" && finding.message.contains("CompileEnvironment")
        }),
        "{:#?}",
        report.findings
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.id == "RUST-MACRO-001")
    );
    reset(&root);
}

fn repository(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "zrail-compile-effects-{name}-{}-{:?}",
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
    write(&root, "zrail.toml", CONTRACT);
    root
}

fn check(root: &std::path::Path) -> Report {
    check_repository(root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check compile-effect fixture")
        .report
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
[source.rust]
module_docs = "required"
facades = "allow"
tests = "allow"
[source.rust.macros]
mode = "deny-unreviewed"
[source.rust.hygiene]
unsafe = "deny"
lint_suppressions = "allow"
[profiles.restricted.effects]
deny = ["compile-environment", "compile-filesystem"]
[[layer]]
name = "app"
packages = ["fixture"]
profiles = ["restricted"]
reason = "Fixture policy."
"#;
