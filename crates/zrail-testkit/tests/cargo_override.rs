//! Repository-controlled Cargo resolution indirection fails closed as structured evidence.

use std::{fs, path::PathBuf};

use zrail_core::{AnalysisQuality, Finding, ReportStatus};
use zrail_rust::check_repository;

#[test]
fn manifest_and_config_resolution_surfaces_are_rejected() {
    let root = repository("surfaces", OVERRIDDEN_MANIFEST, OVERRIDDEN_CONFIG);

    let report = check(&root);
    let overrides = report
        .findings
        .iter()
        .filter(|finding| finding.id == "CARGO-OVERRIDE-001")
        .collect::<Vec<_>>();

    assert_eq!(overrides.len(), 6, "{:#?}", report.findings);
    assert_path(&overrides, "Cargo.toml");
    assert_path(&overrides, ".cargo/config.toml");
    assert!(overrides.iter().all(|finding| {
        finding.rule == "cargo.resolution-override"
            && finding.analysis == AnalysisQuality::Unresolved
            && finding
                .help
                .as_deref()
                .is_some_and(|help| help.contains("remove the override"))
    }));
    reset(&root);
}

#[test]
fn unrelated_local_cargo_configuration_remains_supported() {
    let root = repository("ordinary", MANIFEST, ORDINARY_CONFIG);

    let report = check(&root);

    assert_eq!(report.status, ReportStatus::Pass, "{:#?}", report.findings);
    reset(&root);
}

#[test]
fn named_registry_without_an_attested_index_is_rejected() {
    let root = repository("named-registry", NAMED_REGISTRY_MANIFEST, ORDINARY_CONFIG);

    let report = check(&root);

    assert!(report.findings.iter().any(|finding| {
        finding.id == "CARGO-OVERRIDE-001"
            && finding.path.as_deref() == Some("Cargo.toml")
            && finding.message.contains("named Cargo registry")
    }));
    reset(&root);
}

#[test]
fn included_configuration_is_rejected_without_following_its_paths() {
    for (name, include) in [
        ("string-include", "include = [\"resolution.toml\"]\n"),
        (
            "table-include",
            "include = [{ path = \"resolution.toml\" }]\n",
        ),
        (
            "optional-include",
            "include = [{ path = \"optional.toml\", optional = true }]\n",
        ),
        ("recursive-include", "include = [\"recursive.toml\"]\n"),
        ("escaping-include", "include = [\"../outside.toml\"]\n"),
    ] {
        let root = repository(name, MANIFEST, include);
        fs::write(
            root.join(".cargo/resolution.toml"),
            "[source.crates-io]\nreplace-with = \"fork\"\n",
        )
        .expect("write included override");
        fs::write(
            root.join(".cargo/recursive.toml"),
            "include = [\"resolution.toml\"]\n",
        )
        .expect("write recursive include");

        let report = check(&root);

        assert!(report.findings.iter().any(|finding| {
            finding.id == "CARGO-OVERRIDE-001"
                && finding.path.as_deref() == Some(".cargo/config.toml")
                && finding.message.contains("includes additional files")
        }));
        reset(&root);
    }
}

#[test]
fn unreadable_root_configuration_has_repository_stable_evidence() {
    let root = repository("unreadable", MANIFEST, ORDINARY_CONFIG);
    fs::write(root.join(".cargo/config.toml"), [0xff]).expect("write non-UTF-8 config");

    let report = check(&root);
    let finding = report
        .findings
        .iter()
        .find(|finding| finding.id == "CARGO-OVERRIDE-001")
        .expect("reject unreadable config");

    assert!(finding.message.contains("bounded UTF-8"));
    assert!(!finding.message.contains(&root.display().to_string()));
    reset(&root);
}

#[test]
fn nested_configuration_is_rejected_without_parsing_unbounded_input() {
    let root = repository("nested", MANIFEST, ORDINARY_CONFIG);
    let nested = root.join("nested/.cargo");
    fs::create_dir_all(&nested).expect("create nested Cargo config directory");
    fs::write(nested.join("config.toml"), [0xff]).expect("write nested config evidence");

    let report = check(&root);

    assert!(report.findings.iter().any(|finding| {
        finding.id == "CARGO-OVERRIDE-001"
            && finding.path.as_deref() == Some("nested/.cargo/config.toml")
            && finding.message.contains("invocation-dependent")
    }));
    reset(&root);
}

#[cfg(unix)]
#[test]
fn symlinked_root_cargo_configuration_cannot_escape_attestation() {
    use std::os::unix::fs::symlink;

    let root = repository("symlink", MANIFEST, ORDINARY_CONFIG);
    let config = root.join(".cargo/config.toml");
    fs::remove_file(&config).expect("remove ordinary config");
    fs::write(root.join("cargo-config-source.toml"), ORDINARY_CONFIG).expect("write target");
    symlink("../cargo-config-source.toml", &config).expect("link Cargo config");

    let report = check(&root);

    assert!(report.findings.iter().any(|finding| {
        finding.id == "CARGO-OVERRIDE-001" && finding.path.as_deref() == Some(".cargo/config.toml")
    }));
    reset(&root);
}

#[cfg(unix)]
#[test]
fn symlinked_cargo_configuration_directory_is_not_traversed() {
    use std::os::unix::fs::symlink;

    let root = repository("symlink-directory", MANIFEST, ORDINARY_CONFIG);
    fs::rename(root.join(".cargo"), root.join("cargo-config-target"))
        .expect("move Cargo config directory");
    symlink("cargo-config-target", root.join(".cargo")).expect("link Cargo config directory");

    let report = check(&root);

    assert!(report.findings.iter().any(|finding| {
        finding.id == "CARGO-OVERRIDE-001"
            && finding.path.as_deref() == Some(".cargo")
            && finding.message.contains("repository-local directory")
    }));
    reset(&root);
}

fn assert_path(findings: &[&Finding], path: &str) {
    assert!(
        findings
            .iter()
            .any(|finding| finding.path.as_deref() == Some(path)),
        "missing override at {path}"
    );
}

fn check(root: &std::path::Path) -> zrail_core::Report {
    check_repository(
        root,
        std::path::Path::new("zrail.toml"),
        std::path::Path::new("zrail.lock"),
    )
    .expect("analyze Cargo override fixture")
    .report
}

fn repository(name: &str, manifest: &str, config: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "zrail-override-{name}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create source");
    fs::create_dir_all(root.join(".cargo")).expect("create Cargo config directory");
    fs::write(root.join("Cargo.toml"), manifest).expect("write manifest");
    fs::write(root.join(".cargo/config.toml"), config).expect("write Cargo config");
    fs::write(root.join("src/lib.rs"), "//! Cargo override fixture.\n").expect("write source");
    fs::write(root.join("zrail.toml"), CONTRACT).expect("write contract");
    root
}

fn reset(root: &PathBuf) {
    if root.exists() {
        fs::remove_dir_all(root).expect("reset fixture");
    }
}

const MANIFEST: &str = "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n";

const OVERRIDDEN_MANIFEST: &str = r#"[package]
name = "fixture"
version = "0.0.0"
edition = "2024"

[patch.crates-io]
uuid = { git = "https://example.test/uuid" }

[replace]
"uuid:1.0.0" = { path = "vendor/uuid" }
"#;

const NAMED_REGISTRY_MANIFEST: &str = r#"[package]
name = "fixture"
version = "0.0.0"
edition = "2024"

[dependencies]
private = { version = "1", registry = "private" }
"#;

const OVERRIDDEN_CONFIG: &str = r#"paths = ["vendor"]

[source.crates-io]
replace-with = "mirror"

[registries.private]
index = "https://example.test/index"

[registry]
default = "private"
"#;

const ORDINARY_CONFIG: &str = r#"[build]
target-dir = "target"

[net]
offline = true

[registry]
global-credential-providers = ["cargo:token"]

[registries.private]
credential-provider = "cargo:token"
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

[source.rust.hygiene]
unsafe = "deny"
lint_suppressions = "allow"
deny_methods = []
deny_macros = []
"#;
