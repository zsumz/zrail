//! Init applies canonical exclusions before discovery and preserves atomic baseline writes.

use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::app::{
    args::{InitOptions, InitPreset},
    commands::init,
};

static FIXTURE_SEQUENCE: AtomicUsize = AtomicUsize::new(0);

#[test]
fn exclusion_flags_and_files_render_one_canonical_selection() {
    let root = fixture_root("canonical");
    write_package(&root);
    write_malformed_fixture(&root);
    fs::write(
        root.join(".zrailignore"),
        "# local non-authoritative inputs\n./fixtures/**\ngenerated/**\n",
    )
    .expect("write exclusion file");

    let result = init(&options(
        &root,
        false,
        vec!["generated/**/".into(), "fixtures/**".into()],
        vec![".zrailignore".into()],
    ))
    .expect("initialize selected repository");
    let contract = fs::read_to_string(root.join("zrail.toml")).expect("read contract");

    assert_eq!(result.exit_code, 0, "{}", result.text);
    assert!(result.text.contains("Adoption: contract only"));
    assert!(result.text.contains("Exclusions: 2"));
    assert!(contract.contains("exclude = [\"fixtures/**\", \"generated/**\"]"));
    assert!(!root.join("zrail.lock").exists());
    reset(&root);
}

#[test]
fn baseline_skips_excluded_malformed_fixtures_and_writes_a_lock() {
    let root = fixture_root("baseline");
    write_package(&root);
    write_malformed_fixture(&root);

    let result = init(&options(
        &root,
        true,
        vec!["fixtures/**".into()],
        Vec::new(),
    ))
    .expect("initialize selected baseline");

    assert_eq!(result.exit_code, 0, "{}", result.text);
    assert!(root.join("zrail.toml").is_file());
    assert!(root.join("zrail.lock").is_file());
    reset(&root);
}

#[test]
fn hidden_cargo_target_is_rejected_without_partial_architecture() {
    let root = fixture_root("hidden-target");
    write_package(&root);

    let error = init(&options(
        &root,
        false,
        vec!["src/lib.rs".into()],
        Vec::new(),
    ))
    .expect_err("target exclusion must fail");

    assert!(error.message.contains("authoritative Cargo Library target"));
    assert!(error.message.contains("src/lib.rs"));
    assert!(!root.join("zrail.toml").exists());
    assert!(!root.join("zrail.lock").exists());
    reset(&root);
}

fn options(
    root: &Path,
    baseline: bool,
    exclusions: Vec<String>,
    exclusion_files: Vec<PathBuf>,
) -> InitOptions {
    InitOptions {
        root: root.to_path_buf(),
        preset: InitPreset::Rust,
        baseline,
        exclusions,
        exclusion_files,
    }
}

fn write_package(root: &Path) {
    fs::create_dir_all(root.join("src")).expect("create package");
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = 'selected'\nversion = '0.0.0'\nedition = '2024'\n",
    )
    .expect("write manifest");
    fs::write(root.join("src/lib.rs"), "//! selected package\n").expect("write source");
}

fn write_malformed_fixture(root: &Path) {
    fs::create_dir_all(root.join("fixtures/broken/src")).expect("create fixture");
    fs::write(root.join("fixtures/broken/Cargo.toml"), "not = [valid")
        .expect("write invalid manifest");
    fs::write(
        root.join("fixtures/broken/src/lib.rs"),
        "fn not valid Rust {",
    )
    .expect("write invalid source");
}

fn fixture_root(name: &str) -> PathBuf {
    let sequence = FIXTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "zrail-init-selection-{}-{sequence}-{name}",
        std::process::id()
    ))
}

fn reset(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).expect("reset fixture");
    }
}
