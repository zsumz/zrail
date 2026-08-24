//! Existing contracts adopt only reviewed measurable debt without partial writes.

use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

use zrail_core::ReportStatus;
use zrail_rust::check_repository;

use crate::app::{
    args::{BaselineOptions, CommonOptions, InitOptions, InitPreset},
    commands::{baseline, init},
    output::OutputFormat,
};

static FIXTURE_SEQUENCE: AtomicUsize = AtomicUsize::new(0);

#[test]
fn dry_run_and_refusal_leave_existing_contract_and_missing_lock_unchanged() {
    let root = prepared_repository("dry-run");
    grow_source(&root);
    let original = add_heading(&root);

    let dry_run = run(&root, true, false, OutputFormat::Json).expect("plan baseline");
    assert_eq!(dry_run.exit_code, 0, "{}", dry_run.text);
    assert!(dry_run.text.contains("\"status\": \"dry-run\""));
    assert!(dry_run.text.contains("\"rule\":\"rust.file-size\""));
    assert_eq!(read_contract(&root), original);
    assert!(!root.join("zrail.lock").exists());

    let refused = run(&root, false, false, OutputFormat::Human).expect("refuse grants");
    assert_eq!(refused.exit_code, 1);
    assert!(refused.text.contains("--accept-grants"));
    assert_eq!(read_contract(&root), original);
    assert!(!root.join("zrail.lock").exists());
    reset(&root);
}

#[test]
fn accepted_baseline_preserves_comments_and_second_run_is_byte_idempotent() {
    let root = prepared_repository("accepted");
    grow_source(&root);
    let original = add_heading(&root);

    let accepted = run(&root, false, true, OutputFormat::Human).expect("accept baseline");
    assert_eq!(accepted.exit_code, 0, "{}", accepted.text);
    let contract = read_contract(&root);
    let lock = fs::read_to_string(root.join("zrail.lock")).expect("read lock");
    assert!(contract.starts_with("# hand-authored contract\n"));
    assert!(contract.contains("rule = \"rust.file-size\""));
    assert!(contract.contains("Observed by `zrail baseline`"));
    assert_ne!(contract, original);
    assert_ready(&root);

    let repeated = run(&root, false, false, OutputFormat::Json).expect("repeat baseline");
    assert_eq!(repeated.exit_code, 0, "{}", repeated.text);
    assert!(repeated.text.contains("\"added\": []"));
    assert!(repeated.text.contains("\"preserved\":"));
    assert_eq!(read_contract(&root), contract);
    assert_eq!(fs::read_to_string(root.join("zrail.lock")).unwrap(), lock);
    reset(&root);
}

#[test]
fn non_ratchetable_failure_leaves_both_architecture_files_unchanged() {
    let root = prepared_repository("rejected");
    grow_source(&root);
    fs::write(root.join("src/orphan_test.rs"), "//! unreachable test\n")
        .expect("write orphan test");
    let original = add_heading(&root);

    let rejected = run(&root, false, true, OutputFormat::Json).expect("reject baseline");

    assert_eq!(rejected.exit_code, 1);
    assert!(rejected.text.contains("\"status\": \"rejected\""));
    assert!(rejected.text.contains("RUST-GRAPH-004"));
    assert_eq!(read_contract(&root), original);
    assert!(!root.join("zrail.lock").exists());
    reset(&root);
}

#[test]
fn lock_write_failure_rolls_back_the_contract() {
    let root = prepared_repository("rollback");
    grow_source(&root);
    let original = add_heading(&root);
    fs::create_dir(root.join("zrail.lock")).expect("block lock output");

    let error =
        run(&root, false, true, OutputFormat::Human).expect_err("lock replacement must fail");

    assert!(error.message.contains("regular file"));
    assert_eq!(read_contract(&root), original);
    assert!(root.join("zrail.lock").is_dir());
    reset(&root);
}

fn prepared_repository(name: &str) -> PathBuf {
    let root = fixture_root(name);
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create source");
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"baseline\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("write manifest");
    fs::write(root.join("src/lib.rs"), "//! package\n").expect("write source");
    let initialized = init(&InitOptions {
        root: root.clone(),
        preset: InitPreset::Zsumz,
        baseline: false,
        exclusions: Vec::new(),
        exclusion_files: Vec::new(),
    })
    .expect("initialize contract");
    assert_eq!(initialized.exit_code, 0, "{}", initialized.text);
    assert!(!root.join("zrail.lock").exists());
    root
}

fn run(
    root: &Path,
    dry_run: bool,
    accept_grants: bool,
    format: OutputFormat,
) -> Result<super::CommandResult, crate::app::error::CliError> {
    baseline(&BaselineOptions {
        common: CommonOptions {
            root: root.to_path_buf(),
            format,
            ..CommonOptions::default()
        },
        dry_run,
        accept_grants,
        rule: None,
    })
}

fn grow_source(root: &Path) {
    let source = format!("//! package\n{}", "// legacy\n".repeat(301));
    fs::write(root.join("src/lib.rs"), source).expect("grow source");
}

fn add_heading(root: &Path) -> String {
    let contract = format!("# hand-authored contract\n{}", read_contract(root));
    fs::write(root.join("zrail.toml"), &contract).expect("add comment");
    contract
}

fn read_contract(root: &Path) -> String {
    fs::read_to_string(root.join("zrail.toml")).expect("read contract")
}

fn assert_ready(root: &Path) {
    let result = check_repository(root, Path::new("zrail.toml"), Path::new("zrail.lock"))
        .expect("check baseline");
    assert_eq!(
        result.report.status,
        ReportStatus::Pass,
        "{}",
        result.report.human()
    );
}

fn fixture_root(name: &str) -> PathBuf {
    let sequence = FIXTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "zrail-existing-baseline-{}-{sequence}-{name}",
        std::process::id()
    ))
}

fn reset(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).expect("reset fixture");
    }
}
