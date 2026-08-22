//! Exact per-file size ratchets replace, rather than inflate, class ceilings.

use std::{fs, path::Path};

use zrail_core::ReportStatus;
use zrail_rust::{build_lock, check_repository};

#[test]
fn locked_facade_ratchet_is_the_files_exact_ceiling_until_target() {
    let root = repository("lifecycle", &source(996, "pub fn run() {}"), true);
    lock(&root);

    let exact = check(&root);
    assert_eq!(exact.status, ReportStatus::Pass, "{}", exact.human());

    write_source(&root, &source(997, "pub fn run() {}"));
    let grown = check(&root);
    assert_one(&grown, "RUST-SIZE-003");
    assert_none(&grown, "RUST-SIZE-001");

    write_source(&root, &source(950, "pub fn run() {}"));
    let shrunk = check(&root);
    assert_one(&shrunk, "RUST-SIZE-004");
    assert_none(&shrunk, "RUST-SIZE-001");

    lock(&root);
    write_source(&root, &source(951, "pub fn run() {}"));
    let regrown = check(&root);
    assert_one(&regrown, "RUST-SIZE-003");
    assert_none(&regrown, "RUST-SIZE-001");

    write_source(&root, &source(199, "pub fn run() {}"));
    let at_target = check(&root);
    let stale = finding(&at_target, "RUST-SIZE-004");
    assert!(stale.message.contains("design target"), "{}", stale.message);
    assert_none(&at_target, "RUST-SIZE-001");
    reset(&root);
}

#[test]
fn unratcheted_file_keeps_the_class_hard_ceiling() {
    let root = repository("hard", &source(20, "mod fresh;"), false);
    fs::write(root.join("src/fresh.rs"), source(226, "pub fn run() {}")).expect("write new module");
    lock(&root);

    let report = check(&root);
    let hard = finding(&report, "RUST-SIZE-001");

    assert_eq!(hard.path.as_deref(), Some("src/fresh.rs"));
    assert!(hard.message.contains("225-line ceiling"));
    reset(&root);
}

fn repository(name: &str, contents: &str, ratcheted: bool) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!(
        "zrail-size-ratchet-{name}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create source root");
    fs::write(root.join("Cargo.toml"), MANIFEST).expect("write manifest");
    write_source(&root, contents);
    let ratchet = if ratcheted {
        r#"
[[ratchet]]
rule = "rust.file-size"
target = "src/lib.rs"
reason = "Legacy facade debt must only shrink."
"#
    } else {
        ""
    };
    fs::write(root.join("zrail.toml"), format!("{CONTRACT}{ratchet}")).expect("write contract");
    root
}

fn source(lines: usize, item: &str) -> String {
    assert!(lines >= 2);
    std::iter::once("//! Fixture module.")
        .chain(std::iter::once(item))
        .chain(std::iter::repeat_n("// Legacy line.", lines - 2))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

fn write_source(root: &Path, contents: &str) {
    fs::write(root.join("src/lib.rs"), contents).expect("write source");
}

fn lock(root: &Path) {
    build_lock(root, "zrail.toml".as_ref())
        .expect("build fixture lock")
        .write(&root.join("zrail.lock"))
        .expect("write fixture lock");
}

fn check(root: &Path) -> zrail_core::Report {
    check_repository(root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check fixture")
        .report
}

fn finding<'a>(report: &'a zrail_core::Report, id: &str) -> &'a zrail_core::Finding {
    report
        .findings
        .iter()
        .find(|finding| finding.id == id)
        .unwrap_or_else(|| panic!("missing {id}: {}", report.human()))
}

fn assert_one(report: &zrail_core::Report, id: &str) {
    assert_eq!(
        report
            .findings
            .iter()
            .filter(|finding| finding.id == id)
            .count(),
        1,
        "{}",
        report.human()
    );
}

fn assert_none(report: &zrail_core::Report, id: &str) {
    assert!(
        report.findings.iter().all(|finding| finding.id != id),
        "{}",
        report.human()
    );
}

fn reset(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).expect("reset fixture");
    }
}

const MANIFEST: &str = "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n";

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
module_docs = "allow"
facades = "allow"
tests = "allow"
[source.rust.macros]
mode = "allow"
[source.rust.hygiene]
unsafe = "allow"
lint_suppressions = "allow"
deny_methods = []
deny_macros = []
[source.rust.size.facade]
target = 200
hard = 225
[source.rust.size.implementation]
target = 200
hard = 225
[source.rust.size.test]
target = 200
hard = 225
[source.rust.size.auxiliary]
target = 200
hard = 225
"#;
