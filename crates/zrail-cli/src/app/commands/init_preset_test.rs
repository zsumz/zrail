//! Rust preset initialization preserves conventional Cargo test organization.

use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

use zrail_core::{LockFile, ReportStatus};
use zrail_rust::{check_repository, explain_path};

use crate::app::args::{InitOptions, InitPreset};
use crate::app::commands::{CommandResult, init};

static FIXTURE_SEQUENCE: AtomicUsize = AtomicUsize::new(0);

#[test]
fn rust_preset_accepts_inline_and_integration_tests_without_a_size_policy() {
    let root = fixture_root("strict");
    reset(&root);
    write_conventional_package(&root);

    let result = initialize(&root, false).expect("initialize Rust preset");
    let mut contract = fs::read_to_string(root.join("zrail.toml")).expect("read contract");

    assert_eq!(result.exit_code, 0, "{}", result.text);
    assert!(result.text.contains("Preset: rust"));
    assert!(result.text.contains("Adoption: contract only"));
    assert!(contract.contains("tests = \"allow\""));
    assert!(!contract.contains("[source.rust.size"));
    assert!(!root.join("zrail.lock").exists());
    let explanation = explain_path(&root, Path::new("zrail.toml"), Path::new("src/lib.rs"))
        .expect("explain unbounded Rust source");
    assert_eq!(explanation.schema, 2);
    assert_eq!(explanation.design_target, None);
    assert_eq!(explanation.hard_ceiling, None);
    assert_eq!(explanation.expected_sibling_test, None);
    assert_eq!(explanation.macro_expansion, "allow");
    assert!(explanation.allowed_macro_expansions.is_empty());
    assert!(!explanation.sibling_tests_required);
    assert!(explanation.human().contains("target <not enforced>"));
    assert!(explanation.human().contains("macro expansion: allow"));
    contract.push_str(
        "\n[[ratchet]]\nrule = \"rust.file-size\"\ntarget = \"src/lib.rs\"\nreason = \"stale\"\n",
    );
    fs::write(root.join("zrail.toml"), contract).expect("write stale ratchet");
    let stale = check_repository(
        root.as_path(),
        Path::new("zrail.toml"),
        Path::new("zrail.lock"),
    )
    .expect_err("stale size ratchet must fail contract validation");
    assert!(stale.to_string().contains("no handwritten or generated"));
    reset(&root);
}

#[test]
fn baseline_is_an_independent_no_op_for_allowed_rust_conventions() {
    let root = fixture_root("baseline");
    reset(&root);
    write_conventional_package(&root);

    let result = initialize(&root, true).expect("initialize Rust baseline");
    let contract = fs::read_to_string(root.join("zrail.toml")).expect("read contract");

    assert_eq!(result.exit_code, 0, "{}", result.text);
    assert!(result.text.contains("Adoption: baseline"));
    assert!(result.text.contains("Recorded debt: 0 ratchets"));
    assert!(!contract.contains("[[ratchet]]"));
    assert_ready(&root);
    reset(&root);
}

#[test]
fn rust_preset_accepts_an_external_root_that_no_active_policy_relies_on() {
    let root = fixture_root("external-dependency");
    reset(&root);
    write_conventional_package(&root);
    let manifest = fs::read_to_string(root.join("Cargo.toml")).expect("read manifest");
    fs::write(
        root.join("Cargo.toml"),
        format!("{manifest}\n[dependencies]\nserde = \"1\"\n"),
    )
    .expect("add external dependency");

    let result = initialize(&root, true).expect("initialize dependency-bearing package");
    let lock = LockFile::read(&root.join("zrail.lock")).expect("read initialized lock");
    let dependency = &lock.packages[0].dependencies[0];

    assert_eq!(result.exit_code, 0, "{}", result.text);
    assert_eq!(dependency.name, "serde");
    assert_eq!(dependency.crate_root, None);
    assert_ready(&root);
    reset(&root);
}

fn initialize(root: &Path, baseline: bool) -> Result<CommandResult, crate::app::error::CliError> {
    init(&InitOptions {
        root: root.to_path_buf(),
        preset: InitPreset::Rust,
        baseline,
        exclusions: Vec::new(),
        exclusion_files: Vec::new(),
    })
}

fn write_conventional_package(root: &Path) {
    fs::create_dir_all(root.join("src")).expect("create source");
    fs::create_dir_all(root.join("tests")).expect("create integration tests");
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"conventional\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("write manifest");
    let mut source = String::from("//! conventional package\n");
    source.push_str(&"// ordinary source line\n".repeat(301));
    source.push_str("#[cfg(test)]\nmod tests {\n    #[test]\n    fn works() {}\n}\n");
    fs::write(root.join("src/lib.rs"), source).expect("write library");
    fs::write(
        root.join("tests/integration.rs"),
        "#[test]\nfn public_surface_works() {}\n",
    )
    .expect("write integration test");
}

fn assert_ready(root: &Path) {
    let report = report(root);
    assert_eq!(report.status, ReportStatus::Pass, "{}", report.human());
}

fn report(root: &Path) -> zrail_core::Report {
    check_repository(root, Path::new("zrail.toml"), Path::new("zrail.lock"))
        .expect("check initialized repository")
        .report
}

fn fixture_root(name: &str) -> PathBuf {
    let sequence = FIXTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "zrail-init-preset-{}-{sequence}-{name}",
        std::process::id()
    ))
}

fn reset(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).expect("reset fixture");
    }
}
