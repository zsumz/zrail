//! Repository selection is canonical and deliberately has no negation language.

use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

use super::RepositorySelection;
use crate::onboarding::discover_source_roots_with_selection;

static FIXTURE_SEQUENCE: AtomicUsize = AtomicUsize::new(0);

#[test]
fn exclusions_are_normalized_sorted_and_deduplicated() {
    let selection = RepositorySelection::new([
        "generated/**/".into(),
        "./fixtures/**".into(),
        "generated/**".into(),
    ])
    .expect("normalize exclusions");

    assert_eq!(selection.exclusions(), ["fixtures/**", "generated/**"]);
    assert_eq!(
        selection.matching_exclusion("fixtures/broken/Cargo.toml"),
        Some("fixtures/**")
    );
}

#[test]
fn negation_and_parent_traversal_are_rejected() {
    let negation =
        RepositorySelection::new(["!fixtures/**".into()]).expect_err("negation must fail");
    assert!(negation.to_string().contains("unsupported negation"));

    let escape = RepositorySelection::new(["../fixtures/**".into()])
        .expect_err("parent traversal must fail");
    assert!(escape.to_string().contains("escapes repository"));
}

#[test]
fn excluded_malformed_fixture_manifest_is_not_parsed() {
    let root = fixture_root("malformed-fixture");
    write_package(&root);
    fs::create_dir_all(root.join("fixtures/broken")).expect("create fixture");
    fs::write(root.join("fixtures/broken/Cargo.toml"), "not = [valid")
        .expect("write malformed fixture manifest");
    let selection = RepositorySelection::new(["fixtures/**".into()]).expect("select repository");

    let roots =
        discover_source_roots_with_selection(&root, &selection).expect("ignore excluded fixture");

    assert_eq!(roots, ["."]);
    reset(&root);
}

#[test]
fn exclusion_cannot_hide_an_authoritative_cargo_target() {
    let root = fixture_root("hidden-target");
    write_package(&root);
    let selection = RepositorySelection::new(["src/lib.rs".into()]).expect("select repository");

    let error = discover_source_roots_with_selection(&root, &selection)
        .expect_err("active target must remain selected");

    assert!(
        error
            .to_string()
            .contains("authoritative Cargo Library target")
    );
    assert!(error.to_string().contains("src/lib.rs"));
    reset(&root);
}

fn write_package(root: &Path) {
    fs::create_dir_all(root.join("src")).expect("create package");
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = 'selection'\nversion = '0.0.0'\nedition = '2024'\n",
    )
    .expect("write manifest");
    fs::write(root.join("src/lib.rs"), "//! package\n").expect("write source");
}

fn fixture_root(name: &str) -> PathBuf {
    let sequence = FIXTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "zrail-selection-{}-{sequence}-{name}",
        std::process::id()
    ))
}

fn reset(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).expect("reset fixture");
    }
}
