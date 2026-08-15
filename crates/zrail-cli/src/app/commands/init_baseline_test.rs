//! Baseline initialization records exact debt without relaxing strict source intent.

use std::{
    fmt::Write as _,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

use zrail_core::ReportStatus;
use zrail_rust::check_repository;

use crate::app::{
    args::{InitMode, InitOptions},
    commands::{CommandResult, init},
    error::CliError,
};

static FIXTURE_SEQUENCE: AtomicUsize = AtomicUsize::new(0);

#[test]
fn baseline_records_size_and_inline_test_debt_as_ratchets() {
    let root = fixture_root("record-debt");
    reset(&root);
    write_package(&root, &oversized_inline_source(301, 1));

    let result = initialize(&root).expect("initialize baseline");
    assert_eq!(result.exit_code, 0, "{}", result.text);
    let contract = fs::read_to_string(root.join("zrail.toml")).expect("read contract");
    let lock = fs::read_to_string(root.join("zrail.lock")).expect("read lock");

    assert!(result.text.contains("Mode: baseline"));
    assert!(result.text.contains("Recorded debt: 2 ratchets"));
    assert!(contract.contains("tests = \"sibling\""));
    assert_eq!(contract.matches("target = 300").count(), 4);
    assert!(contract.contains("rule = \"rust.file-size\""));
    assert!(contract.contains("rule = \"rust.inline-tests\""));
    assert_eq!(lock.matches("target = \"src/lib.rs\"").count(), 2);
    assert_ready(&root);
    reset(&root);
}

#[test]
fn baseline_inline_test_debt_cannot_grow_or_shrink_silently() {
    let root = fixture_root("tighten-inline");
    reset(&root);
    write_package(&root, &oversized_inline_source(0, 2));
    let initialized = initialize(&root).expect("initialize baseline");
    assert_eq!(initialized.exit_code, 0, "{}", initialized.text);

    fs::write(root.join("src/lib.rs"), oversized_inline_source(0, 3)).expect("grow inline tests");
    let grown = report(&root);
    assert_eq!(grown.status, ReportStatus::Fail);
    assert!(
        grown
            .findings
            .iter()
            .any(|finding| finding.id == "RUST-TEST-005")
    );

    fs::write(root.join("src/lib.rs"), oversized_inline_source(0, 1)).expect("shrink inline tests");
    let shrunk = report(&root);
    assert_eq!(shrunk.status, ReportStatus::Fail);
    assert!(
        shrunk
            .findings
            .iter()
            .any(|finding| { finding.id == "RUST-TEST-005" && finding.message.contains("shrank") })
    );
    reset(&root);
}

#[test]
fn debt_free_baseline_retains_the_strict_template() {
    let root = fixture_root("debt-free");
    reset(&root);
    write_package(&root, "//! package\n");

    let result = initialize(&root).expect("initialize debt-free baseline");
    let contract = fs::read_to_string(root.join("zrail.toml")).expect("read contract");

    assert!(result.text.contains("Recorded debt: 0 ratchets"));
    assert!(!contract.contains("[[ratchet]]"));
    assert_eq!(contract.matches("hard = 300").count(), 4);
    assert_ready(&root);
    reset(&root);
}

#[test]
fn baseline_refuses_unratchetable_source_debt_without_partial_state() {
    let root = fixture_root("unratchetable");
    reset(&root);
    write_package(&root, "//! package\n");
    fs::write(root.join("src/orphan_test.rs"), "//! orphan test\n")
        .expect("write unreachable test");

    let result = initialize(&root).expect("evaluate baseline");

    assert_eq!(result.exit_code, 1);
    assert!(result.text.contains("RUST-GRAPH-004"));
    assert!(!root.join("zrail.toml").exists());
    assert!(!root.join("zrail.lock").exists());
    reset(&root);
}

fn initialize(root: &Path) -> Result<CommandResult, CliError> {
    init(&InitOptions {
        root: root.to_path_buf(),
        mode: InitMode::Baseline,
    })
}

fn report(root: &Path) -> zrail_core::Report {
    check_repository(root, Path::new("zrail.toml"), Path::new("zrail.lock"))
        .expect("check baseline repository")
        .report
}

fn assert_ready(root: &Path) {
    let report = report(root);
    assert_eq!(report.status, ReportStatus::Pass, "{}", report.human());
}

fn oversized_inline_source(comment_lines: usize, tests: usize) -> String {
    let mut source = String::from("//! package\n");
    source.push_str(&"// legacy line\n".repeat(comment_lines));
    source.push_str("#[cfg(test)]\nmod tests {\n");
    for index in 0..tests {
        let _ = write!(source, "    #[test]\n    fn proof_{index}() {{}}\n");
    }
    source.push_str("}\n");
    source
}

fn write_package(root: &Path, source: &str) {
    fs::create_dir_all(root.join("src")).expect("create package");
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"baseline\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("write manifest");
    fs::write(root.join("src/lib.rs"), source).expect("write source");
}

fn fixture_root(name: &str) -> PathBuf {
    let sequence = FIXTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "zrail-init-baseline-{}-{sequence}-{name}",
        std::process::id()
    ))
}

fn reset(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).expect("reset fixture");
    }
}
