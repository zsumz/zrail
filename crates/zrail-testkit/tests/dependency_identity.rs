//! Adversarial Cargo identities remain distinct in observed architecture state.

use std::{fs, path::PathBuf};

use zrail_core::{
    LockedDependency, LockedDependencyScope, LockedDependencySource, compare_architecture,
    load_contract,
};
use zrail_rust::build_lock;

#[test]
fn aliases_sources_targets_and_workspace_inheritance_are_locked() {
    let root = repository("identity");
    let lock = build_lock(&root, std::path::Path::new("zrail.toml")).expect("build exact lock");
    let app = lock
        .packages
        .iter()
        .find(|package| package.name == "app")
        .expect("app package");

    assert_source(app, "internal", LockedDependencyScope::Internal, |source| {
        matches!(
            source,
            LockedDependencySource::WorkspaceMember { directory, .. }
                if directory == "crates/internal"
        )
    });
    assert_source(
        app,
        "remote-internal",
        LockedDependencyScope::External,
        |source| matches!(source, LockedDependencySource::Git { rev, .. } if rev.as_deref() == Some("abc")),
    );
    assert_source(
        app,
        "registry-internal",
        LockedDependencyScope::External,
        |source| matches!(source, LockedDependencySource::Registry { .. }),
    );
    assert_source(
        app,
        "registry-shared",
        LockedDependencyScope::External,
        |source| matches!(source, LockedDependencySource::Registry { .. }),
    );
    assert_source(
        app,
        "git-shared",
        LockedDependencyScope::External,
        |source| matches!(source, LockedDependencySource::Git { .. }),
    );
    assert_source(
        app,
        "excluded",
        LockedDependencyScope::External,
        |source| matches!(source, LockedDependencySource::RepositoryPath { path, .. } if path == "crates/excluded"),
    );
    let inherited = dependency(app, "inherited");
    assert_eq!(inherited.name, "shared");
    assert_eq!(inherited.features, ["member", "root"]);
    assert_eq!(inherited.optional, Some(true));
    let targeted = dependency(app, "targeted");
    assert_eq!(targeted.target.as_deref(), Some("cfg(unix)"));

    reset(&root);
}

#[test]
fn source_and_target_broadening_are_semantic_grants() {
    let root = repository("changes");
    let config = std::path::Path::new("zrail.toml");
    let contract = load_contract(&root, config).expect("load contract");
    let before = build_lock(&root, config).expect("build before lock");
    let manifest = root.join("crates/app/Cargo.toml");
    let source = fs::read_to_string(&manifest).expect("read app manifest");
    let changed = source
        .replace(
            "registry-shared = { package = \"shared\", version = \"1\" }",
            "registry-shared = { package = \"shared\", git = \"https://example.test/shared\" }",
        )
        .replace(
            "[target.'cfg(unix)'.dependencies]\ntargeted = \"1\"",
            "[dependencies.targeted]\nversion = \"1\"",
        );
    fs::write(manifest, changed).expect("broaden dependency identity");
    let after = build_lock(&root, config).expect("build after lock");

    let report = compare_architecture(
        &contract.contract,
        Some(&before),
        &contract.contract,
        Some(&after),
    );

    assert!(report.denies_grants(), "{}", report.human());
    assert!(report.changes.iter().any(|change| {
        change.subject.contains("registry-shared") && change.subject.contains("git:")
    }));
    assert!(report.changes.iter().any(|change| {
        change.subject.contains("targeted") && change.subject.contains("all-targets")
    }));
    reset(&root);
}

fn assert_source(
    package: &zrail_core::LockedPackage,
    alias: &str,
    scope: LockedDependencyScope,
    predicate: impl FnOnce(&LockedDependencySource) -> bool,
) {
    let dependency = dependency(package, alias);
    assert_eq!(dependency.scope, scope);
    assert!(
        dependency.source.as_ref().is_some_and(predicate),
        "{}",
        dependency.label()
    );
}

fn dependency<'a>(package: &'a zrail_core::LockedPackage, alias: &str) -> &'a LockedDependency {
    package
        .dependencies
        .iter()
        .find(|dependency| dependency.alias.as_deref() == Some(alias))
        .unwrap_or_else(|| panic!("missing dependency alias {alias:?}"))
}

fn repository(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "zrail-dependency-{name}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    for package in ["app", "internal", "excluded"] {
        fs::create_dir_all(root.join(format!("crates/{package}/src"))).expect("create package");
        fs::write(
            root.join(format!("crates/{package}/src/lib.rs")),
            format!("//! {package}\n"),
        )
        .expect("write source");
    }
    fs::write(root.join("Cargo.toml"), ROOT_MANIFEST).expect("write root manifest");
    fs::write(root.join("crates/app/Cargo.toml"), APP_MANIFEST).expect("write app manifest");
    fs::write(
        root.join("crates/internal/Cargo.toml"),
        package_manifest("internal"),
    )
    .expect("write internal manifest");
    fs::write(
        root.join("crates/excluded/Cargo.toml"),
        package_manifest("excluded"),
    )
    .expect("write excluded manifest");
    fs::write(root.join("zrail.toml"), CONTRACT).expect("write contract");
    root
}

fn package_manifest(name: &str) -> String {
    format!("[package]\nname = \"{name}\"\nversion = \"0.0.0\"\nedition = \"2024\"\n")
}

fn reset(root: &PathBuf) {
    if root.exists() {
        fs::remove_dir_all(root).expect("reset fixture");
    }
}

const ROOT_MANIFEST: &str = r#"[workspace]
members = ["crates/app", "crates/internal"]
exclude = ["crates/excluded"]
resolver = "3"

[workspace.dependencies]
internal = { path = "crates/internal" }
inherited = { package = "shared", version = "1", features = ["root"] }
"#;

const APP_MANIFEST: &str = r#"[package]
name = "app"
version = "0.0.0"
edition = "2024"

[dependencies]
internal.workspace = true
remote-internal = { package = "internal", git = "https://example.test/internal", rev = "abc" }
registry-internal = { package = "internal", version = "1" }
registry-shared = { package = "shared", version = "1" }
git-shared = { package = "shared", git = "https://example.test/shared", tag = "v1" }
alias-one = { package = "aliased", version = "1" }
alias-two = { package = "aliased", git = "https://example.test/aliased" }
inherited = { workspace = true, optional = true, features = ["member"] }
excluded = { path = "../excluded" }

[target.'cfg(unix)'.dependencies]
targeted = "1"
"#;

const CONTRACT: &str = r#"schema = 1
adapters = ["rust"]

[repository]
roots = ["crates"]
exclude = []
workspace_members = "exact"
nested_git = "deny"
submodules = "deny"
symlinks = "inside"

[dependencies]
mode = "locked"
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
