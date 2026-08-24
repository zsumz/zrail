//! Resolved Cargo identity is source-aware and fails closed on multi-version ambiguity.

use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

use super::{ResolvedCargoGraph, validate_resolved_sources};
use crate::cargo::{CrateRootAuthority, Dependency, DependencyKind, DependencySource, Package};
use zrail_core::load_contract;

static FIXTURE_SEQUENCE: AtomicUsize = AtomicUsize::new(0);
const REGISTRY: &str = "registry+https://github.com/rust-lang/crates.io-index";

#[test]
fn exact_identity_retains_version_source_and_checksum() {
    let root = fixture_root("identity");
    write_lock(
        &root,
        &format!(
            r#"version = 4

[[package]]
name = "app"
version = "0.1.0"
dependencies = ["bridge"]

[[package]]
name = "bridge"
version = "1.2.3"
source = "{REGISTRY}"
checksum = "{}"
"#,
            checksum('a')
        ),
    );

    let graph = ResolvedCargoGraph::load(&root, &[package(vec![registry("1")])])
        .expect("load graph")
        .expect("Cargo.lock exists");
    let identity = graph.lookup("bridge", None, None).expect("unique bridge");
    let expected_checksum = checksum('a');

    assert_eq!(identity.version, "1.2.3");
    assert_eq!(identity.source, REGISTRY);
    assert_eq!(
        identity.checksum.as_deref(),
        Some(expected_checksum.as_str())
    );
    reset(&root);
}

#[test]
fn manifest_requirement_disambiguates_multiple_locked_versions() {
    let root = fixture_root("versions");
    write_multiversion_lock(&root);
    let package = package(vec![registry("1")]);
    let graph = ResolvedCargoGraph::load(&root, std::slice::from_ref(&package))
        .expect("load graph")
        .expect("Cargo.lock exists");

    let target = graph
        .manifest_dependency(&package, &package.dependencies[0])
        .expect("map version-one dependency");
    let version_two = graph
        .lookup("bridge", Some("2.1.0"), Some(REGISTRY))
        .expect("exact version lookup");

    assert_eq!(target.version, "1.4.0");
    assert!(
        graph
            .source_matches(&target, &package.dependencies[0].source)
            .expect("compare selected source")
    );
    assert!(
        !graph
            .source_matches(version_two, &package.dependencies[0].source)
            .expect("reject another resolved version")
    );
    assert!(graph.lookup("bridge", None, None).is_err());
    assert_eq!(version_two.version, "2.1.0");
    reset(&root);
}

#[test]
fn ambiguous_manifest_mapping_fails_closed() {
    let root = fixture_root("ambiguous");
    write_multiversion_lock(&root);
    let package = package(vec![registry("*")]);
    let graph = ResolvedCargoGraph::load(&root, std::slice::from_ref(&package))
        .expect("load graph")
        .expect("Cargo.lock exists");

    let error = graph
        .manifest_dependency(&package, &package.dependencies[0])
        .expect_err("wildcard maps to multiple nodes");

    assert!(error.contains("maps ambiguously"));
    assert!(error.contains("bridge 1.4.0"));
    assert!(error.contains("bridge 2.1.0"));
    reset(&root);
}

#[test]
fn git_identity_requires_a_precise_locked_revision() {
    let root = fixture_root("git-revision");
    write_lock(
        &root,
        r#"version = 4

[[package]]
name = "app"
version = "0.1.0"
dependencies = ["bridge"]

[[package]]
name = "bridge"
version = "1.2.3"
source = "git+https://example.test/bridge#moving-branch"
"#,
    );

    let error = ResolvedCargoGraph::load(&root, &[package(vec![git()])])
        .expect_err("moving Git identity must fail");

    assert!(error.to_string().contains("precise source revision"));
    reset(&root);
}

#[test]
fn cargo_lock_source_selector_requires_exactly_one_node() {
    let root = fixture_root("source-selector");
    write_multiversion_lock(&root);
    let package = package(vec![registry("1")]);
    let graph = ResolvedCargoGraph::load(&root, std::slice::from_ref(&package))
        .expect("load graph")
        .expect("Cargo.lock exists");
    fs::write(
        root.join("zrail.toml"),
        SOURCE_CONTRACT.replace("VERSION", ""),
    )
    .expect("write ambiguous contract");
    let ambiguous = load_contract(&root, Path::new("zrail.toml")).expect("load contract");

    let error = validate_resolved_sources(Some(&graph), &ambiguous.contract)
        .expect_err("package-only selector is ambiguous");

    assert!(error.to_string().contains("ambiguous across 2 nodes"));
    fs::write(
        root.join("zrail.toml"),
        SOURCE_CONTRACT.replace("VERSION", "version = '2.1.0'"),
    )
    .expect("write exact contract");
    let exact = load_contract(&root, Path::new("zrail.toml")).expect("load exact contract");
    validate_resolved_sources(Some(&graph), &exact.contract).expect("selector is exact");
    reset(&root);
}

fn package(dependencies: Vec<Dependency>) -> Package {
    Package {
        name: "app".into(),
        edition: "2024".into(),
        directory: ".".into(),
        dependencies,
        targets: Vec::new(),
    }
}

fn registry(requirement: &str) -> Dependency {
    Dependency {
        alias: "bridge".into(),
        name: "bridge".into(),
        explicit_package: false,
        crate_root: "bridge".into(),
        crate_root_authority: CrateRootAuthority::Unresolved,
        kind: DependencyKind::Normal,
        target: None,
        optional: false,
        default_features: true,
        features: Vec::new(),
        source: DependencySource::Registry {
            registry: None,
            index: None,
            requirement: requirement.into(),
        },
    }
}

fn git() -> Dependency {
    Dependency {
        source: DependencySource::Git {
            repository: "https://example.test/bridge".into(),
            branch: None,
            tag: None,
            rev: None,
            requirement: None,
        },
        ..registry("1")
    }
}

fn write_multiversion_lock(root: &Path) {
    write_lock(
        root,
        &format!(
            r#"version = 4

[[package]]
name = "app"
version = "0.1.0"
dependencies = [
 "bridge 1.4.0 ({REGISTRY})",
 "bridge 2.1.0 ({REGISTRY})",
]

[[package]]
name = "bridge"
version = "1.4.0"
source = "{REGISTRY}"
checksum = "{}"

[[package]]
name = "bridge"
version = "2.1.0"
source = "{REGISTRY}"
checksum = "{}"
"#,
            checksum('1'),
            checksum('2')
        ),
    );
}

fn checksum(character: char) -> String {
    std::iter::repeat_n(character, 64).collect()
}

fn write_lock(root: &Path, source: &str) {
    fs::create_dir_all(root).expect("create fixture");
    fs::write(root.join("Cargo.lock"), source).expect("write Cargo.lock");
}

fn fixture_root(name: &str) -> PathBuf {
    let sequence = FIXTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "zrail-resolved-cargo-{}-{sequence}-{name}",
        std::process::id()
    ))
}

fn reset(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).expect("reset fixture");
    }
}

const SOURCE_CONTRACT: &str = r"schema = 2
adapters = ['rust']

[repository]
roots = ['.']
exclude = []
workspace_members = 'exact'
nested_git = 'deny'
submodules = 'deny'
symlinks = 'inside'

[dependencies]
mode = 'observed'
unassigned_packages = 'allow'
cycles = 'deny'

[source.rust]
module_docs = 'allow'
facades = 'allow'
entrypoints = 'allow'
tests = 'allow'

[source.rust.hygiene]
unsafe = 'deny'
lint_suppressions = 'allow'
deny_methods = []
deny_macros = []

[source.rust.macros]
mode = 'deny-unreviewed'
[[source.rust.macros.allow]]
name = 'bridge::generate'
resolution = 'exact'
namespace_effect = 'none'
reason = 'Reviewed exact lock identity.'
[source.rust.macros.allow.source]
kind = 'cargo-lock'
package = 'bridge'
VERSION
";
