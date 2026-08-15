//! Initialization covers standalone packages and conventional or custom workspaces.

use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

use zrail_core::ReportStatus;
use zrail_rust::check_repository;

use crate::app::args::{InitMode, InitOptions};

use super::init;

static FIXTURE_SEQUENCE: AtomicUsize = AtomicUsize::new(0);

#[test]
fn standalone_package_initializes_at_the_repository_root() {
    let root = fixture_root("standalone");
    reset(&root);
    write_package(&root, "standalone");

    let result = initialize(&root).expect("initialize standalone package");
    let contract = fs::read_to_string(root.join("zrail.toml")).expect("read contract");

    assert_eq!(result.exit_code, 0);
    assert!(contract.contains("roots = [\".\"]"));
    assert_strict_defaults(&contract);
    assert_ready(&root);
    reset(&root);
}

#[test]
fn root_package_workspace_initializes_as_one_repository_boundary() {
    let root = fixture_root("root-workspace");
    reset(&root);
    fs::create_dir_all(&root).expect("create root");
    fs::write(
        root.join("Cargo.toml"),
        concat!(
            "[workspace]\n",
            "members = [\"member\"]\n",
            "resolver = \"3\"\n\n",
            "[package]\n",
            "name = \"root-package\"\n",
            "version = \"0.1.0\"\n",
            "edition = \"2024\"\n\n",
            "[dependencies]\n",
            "member = { path = \"member\" }\n",
        ),
    )
    .expect("write root manifest");
    write_source(&root.join("src"));
    write_package(&root.join("member"), "member");

    let result = initialize(&root).expect("initialize root package workspace");
    let contract = fs::read_to_string(root.join("zrail.toml")).expect("read contract");

    assert_eq!(result.exit_code, 0);
    assert!(contract.contains("roots = [\".\"]"));
    assert_ready(&root);
    reset(&root);
}

#[test]
fn virtual_workspace_discovers_custom_package_directories() {
    let root = fixture_root("custom-workspace");
    reset(&root);
    fs::create_dir_all(&root).expect("create root");
    fs::write(
        root.join("Cargo.toml"),
        concat!(
            "[workspace]\n",
            "members = [\"components/domain\", \"tools/cli\"]\n",
            "resolver = \"3\"\n",
        ),
    )
    .expect("write workspace");
    write_package(&root.join("components/domain"), "domain");
    write_package(&root.join("tools/cli"), "workspace-cli");

    let result = initialize(&root).expect("initialize custom workspace");
    let contract = fs::read_to_string(root.join("zrail.toml")).expect("read contract");

    assert_eq!(result.exit_code, 0);
    assert!(contract.contains("roots = [\"components/domain\", \"tools/cli\"]"));
    assert_ready(&root);
    reset(&root);
}

#[test]
fn strict_test_placement_refuses_inline_test_debt_without_partial_state() {
    let root = fixture_root("inline-tests");
    reset(&root);
    write_package(&root, "inline-tests");
    fs::write(
        root.join("src/lib.rs"),
        "//! package\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn works() {}\n}\n",
    )
    .expect("write inline test");

    let result = initialize(&root).expect("evaluate starter contract");

    assert_eq!(result.exit_code, 1);
    assert!(result.text.contains("RUST-TEST-001"));
    assert!(!root.join("zrail.toml").exists());
    assert!(!root.join("zrail.lock").exists());
    reset(&root);
}

#[test]
fn hard_line_ceiling_refuses_oversized_source_without_partial_state() {
    let root = fixture_root("oversized");
    reset(&root);
    write_package(&root, "oversized");
    let source = std::iter::repeat_n("// line\n", 301).collect::<String>();
    fs::write(root.join("src/lib.rs"), format!("//! package\n{source}"))
        .expect("write oversized source");

    let result = initialize(&root).expect("evaluate starter contract");

    assert_eq!(result.exit_code, 1);
    assert!(result.text.contains("RUST-SIZE-002"));
    assert!(!root.join("zrail.toml").exists());
    assert!(!root.join("zrail.lock").exists());
    reset(&root);
}

#[test]
fn initialization_never_overwrites_existing_architecture() {
    let root = fixture_root("existing");
    reset(&root);
    write_package(&root, "existing");
    fs::write(root.join("zrail.toml"), "sentinel\n").expect("write existing contract");

    let error = initialize(&root).expect_err("existing architecture must be preserved");

    assert!(error.message.contains("never overwrites"));
    assert_eq!(
        fs::read_to_string(root.join("zrail.toml")).expect("read existing contract"),
        "sentinel\n"
    );
    assert!(!root.join("zrail.lock").exists());
    reset(&root);
}

#[test]
fn non_cargo_directories_are_rejected_without_partial_state() {
    let root = fixture_root("non-cargo");
    reset(&root);
    fs::create_dir_all(&root).expect("create directory");

    let error = initialize(&root).expect_err("Cargo metadata is required");

    assert!(error.message.contains("Cargo.toml"));
    assert!(!root.join("zrail.toml").exists());
    assert!(!root.join("zrail.lock").exists());
    reset(&root);
}

fn initialize(root: &Path) -> Result<super::CommandResult, crate::app::error::CliError> {
    init(&InitOptions {
        root: root.to_path_buf(),
        mode: InitMode::Strict,
    })
}

fn assert_strict_defaults(contract: &str) {
    assert!(contract.contains("tests = \"sibling\""));
    assert_eq!(contract.matches("target = 300").count(), 4);
    assert_eq!(contract.matches("hard = 300").count(), 4);
}

fn assert_ready(root: &Path) {
    let report = check_repository(root, Path::new("zrail.toml"), Path::new("zrail.lock"))
        .expect("check initialized repository")
        .report;
    assert_eq!(report.status, ReportStatus::Pass, "{}", report.human());
}

fn write_package(root: &Path, name: &str) {
    fs::create_dir_all(root).expect("create package");
    fs::write(
        root.join("Cargo.toml"),
        format!("[package]\nname = {name:?}\nversion = \"0.1.0\"\nedition = \"2024\"\n"),
    )
    .expect("write package manifest");
    write_source(&root.join("src"));
}

fn write_source(source: &Path) {
    fs::create_dir_all(source).expect("create source");
    fs::write(source.join("lib.rs"), "//! package\n").expect("write source");
}

fn fixture_root(name: &str) -> PathBuf {
    let sequence = FIXTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "zrail-init-{}-{sequence}-{name}",
        std::process::id()
    ))
}

fn reset(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).expect("reset fixture");
    }
}
