//! Manifest Git references must correspond exactly to Cargo.lock source identities.

use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

use super::ResolvedCargoGraph;
use crate::cargo::{CrateRootAuthority, Dependency, DependencyKind, DependencySource, Package};

const REPOSITORY: &str = "https://example.test/bridge";
const COMMIT_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const COMMIT_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
static FIXTURE_SEQUENCE: AtomicUsize = AtomicUsize::new(0);

#[test]
fn manifest_revision_matches_query_and_precise_commit() {
    let source = git_source(&format!("rev={COMMIT_A}"), COMMIT_A);
    let (root, graph, package) = fixture("rev-match", &[&source], Reference::Rev(COMMIT_A));

    let resolved = map(&graph, &package).expect("exact revision maps");

    assert_eq!(resolved.source, source);
    reset(&root);
}

#[test]
fn stale_or_forged_revision_is_rejected() {
    let stale = git_source(&format!("rev={COMMIT_A}"), COMMIT_A);
    let (root, graph, package) = fixture("rev-stale", &[&stale], Reference::Rev(COMMIT_B));
    assert!(map(&graph, &package).is_err());
    reset(&root);

    let forged = git_source(&format!("rev={COMMIT_B}"), COMMIT_A);
    let (root, graph, package) = fixture("rev-forged", &[&forged], Reference::Rev(COMMIT_B));
    assert!(map(&graph, &package).is_err());
    reset(&root);
}

#[test]
fn tag_mismatch_is_rejected() {
    let source = git_source("tag=v1", COMMIT_A);
    let (root, graph, package) = fixture("tag", &[&source], Reference::Tag("v2"));

    assert!(map(&graph, &package).is_err());
    reset(&root);
}

#[test]
fn branch_mismatch_is_rejected() {
    let source = git_source("branch=main", COMMIT_A);
    let (root, graph, package) = fixture("branch", &[&source], Reference::Branch("next"));

    assert!(map(&graph, &package).is_err());
    reset(&root);
}

#[test]
fn encoded_branch_identity_matches_exactly() {
    let source = git_source("branch=feature%2Fnext", COMMIT_A);
    let (root, graph, package) = fixture(
        "encoded-branch",
        &[&source],
        Reference::Branch("feature/next"),
    );

    assert!(map(&graph, &package).is_ok());
    reset(&root);
}

#[test]
fn tag_and_default_branch_match_their_exact_cases() {
    let tagged = git_source("tag=v1", COMMIT_A);
    let (root, graph, package) = fixture("tag-match", &[&tagged], Reference::Tag("v1"));
    assert!(map(&graph, &package).is_ok());
    reset(&root);

    let default = format!("git+{REPOSITORY}#{COMMIT_A}");
    let (root, graph, package) = fixture("default-match", &[&default], Reference::Default);
    assert!(map(&graph, &package).is_ok());
    reset(&root);
}

#[test]
fn default_branch_is_distinct_from_named_branch() {
    let named = git_source("branch=main", COMMIT_A);
    let (root, graph, package) = fixture("default-named", &[&named], Reference::Default);
    assert!(map(&graph, &package).is_err());
    reset(&root);

    let default = format!("git+{REPOSITORY}#{COMMIT_A}");
    let (root, graph, package) = fixture("named-default", &[&default], Reference::Branch("main"));
    assert!(map(&graph, &package).is_err());
    reset(&root);
}

#[test]
fn two_git_nodes_resolve_by_reference_not_repository_alone() {
    let source_a = git_source(&format!("rev={COMMIT_A}"), COMMIT_A);
    let source_b = git_source(&format!("rev={COMMIT_B}"), COMMIT_B);
    let (root, graph, package) = fixture(
        "two-nodes",
        &[&source_a, &source_b],
        Reference::Rev(COMMIT_B),
    );

    let resolved = map(&graph, &package).expect("revision selects one same-repository node");

    assert_eq!(resolved.source, source_b);
    reset(&root);
}

#[test]
fn same_reference_with_two_precise_nodes_is_ambiguous() {
    let source_a = format!("git+{REPOSITORY}#{COMMIT_A}");
    let source_b = format!("git+{REPOSITORY}#{COMMIT_B}");
    let (root, graph, package) =
        fixture("two-defaults", &[&source_a, &source_b], Reference::Default);

    let error = map(&graph, &package).expect_err("same-reference nodes must fail closed");

    assert!(error.contains("maps ambiguously"));
    reset(&root);
}

#[test]
fn named_revision_is_rejected_as_unprovable() {
    let source = git_source("rev=refs%2Fpull%2F1%2Fhead", COMMIT_A);
    let (root, graph, package) =
        fixture("named-rev", &[&source], Reference::Rev("refs/pull/1/head"));

    let error = map(&graph, &package).expect_err("named revision cannot prove commit equality");

    assert!(error.contains("cannot prove correspondence"));
    reset(&root);
}

#[test]
fn unknown_reference_query_fails_closed() {
    let source = git_source("ref=main", COMMIT_A);
    let (root, graph, package) = fixture("unknown-query", &[&source], Reference::Branch("main"));

    let error = map(&graph, &package).expect_err("unknown Cargo query must fail closed");

    assert!(error.contains("unsupported reference kind"));
    reset(&root);
}

#[derive(Clone, Copy)]
enum Reference<'a> {
    Default,
    Branch(&'a str),
    Tag(&'a str),
    Rev(&'a str),
}

fn fixture(
    name: &str,
    sources: &[&str],
    reference: Reference<'_>,
) -> (PathBuf, ResolvedCargoGraph, Package) {
    let root = fixture_root(name);
    write_lock(&root, sources);
    let package = package(reference);
    let graph = ResolvedCargoGraph::load(&root, std::slice::from_ref(&package))
        .expect("load graph")
        .expect("Cargo.lock exists");
    (root, graph, package)
}

fn map(
    graph: &ResolvedCargoGraph,
    package: &Package,
) -> Result<super::ResolvedPackageIdentity, String> {
    graph.manifest_dependency(package, &package.dependencies[0])
}

fn package(reference: Reference<'_>) -> Package {
    let (branch, tag, rev) = match reference {
        Reference::Default => (None, None, None),
        Reference::Branch(value) => (Some(value.into()), None, None),
        Reference::Tag(value) => (None, Some(value.into()), None),
        Reference::Rev(value) => (None, None, Some(value.into())),
    };
    Package {
        name: "app".into(),
        edition: "2024".into(),
        directory: ".".into(),
        dependencies: vec![Dependency {
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
            source: DependencySource::Git {
                repository: REPOSITORY.into(),
                branch,
                tag,
                rev,
                requirement: None,
            },
        }],
        targets: Vec::new(),
    }
}

fn write_lock(root: &Path, sources: &[&str]) {
    let dependencies = sources
        .iter()
        .map(|source| format!(" \"bridge 1.2.3 ({source})\""))
        .collect::<Vec<_>>()
        .join(",\n");
    let packages = sources
        .iter()
        .map(|source| {
            format!("[[package]]\nname = \"bridge\"\nversion = \"1.2.3\"\nsource = \"{source}\"\n")
        })
        .collect::<Vec<_>>()
        .join("\n");
    let lock = format!(
        "version = 4\n\n[[package]]\nname = \"app\"\nversion = \"0.1.0\"\ndependencies = [\n{dependencies},\n]\n\n{packages}"
    );
    fs::create_dir_all(root).expect("create fixture");
    fs::write(root.join("Cargo.lock"), lock).expect("write Cargo.lock");
}

fn git_source(query: &str, precise: &str) -> String {
    format!("git+{REPOSITORY}?{query}#{precise}")
}

fn fixture_root(name: &str) -> PathBuf {
    let sequence = FIXTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "zrail-resolved-git-{}-{sequence}-{name}",
        std::process::id()
    ))
}

fn reset(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).expect("reset fixture");
    }
}
