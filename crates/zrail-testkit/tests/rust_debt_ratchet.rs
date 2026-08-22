//! Module-doc and hygiene debt ratchets preserve strict policy during adoption.

use std::{fs, path::Path};

use zrail_core::ReportStatus;
use zrail_rust::{build_lock, check_repository, discover_baseline};

#[test]
fn module_doc_debt_is_exact_and_new_files_receive_no_exemption() {
    let root = repository(
        "module-docs",
        "mod worker;\npub fn run() {}\n",
        "//! Worker.\npub fn work() {}\n",
        "required",
        "allow",
        "allow",
        &ratchet("rust.module-docs"),
    );
    let missing_lock = finding(&check(&root), "RUST-DOC-002").clone();
    assert!(
        missing_lock.message.contains("absent"),
        "{}",
        missing_lock.message
    );
    lock(&root);
    assert_eq!(check(&root).status, ReportStatus::Pass);

    write(&root, "src/worker.rs", "pub fn work() {}\n");
    let new_debt = check(&root);
    let missing = finding(&new_debt, "RUST-DOC-001");
    assert_eq!(missing.path.as_deref(), Some("src/worker.rs"));

    write(&root, "src/worker.rs", "//! Worker.\npub fn work() {}\n");
    write(
        &root,
        "src/lib.rs",
        "//! Fixture.\nmod worker;\npub fn run() {}\n",
    );
    let cleaned = check(&root);
    let stale = finding(&cleaned, "RUST-DOC-002");
    assert!(stale.message.contains("removed"), "{}", stale.message);
    reset(&root);
}

#[test]
fn unsafe_debt_shrink_and_regrowth_are_both_explicit() {
    let two = "//! Fixture.\npub unsafe fn first() {}\npub fn second() { unsafe {} }\n";
    let one = "//! Fixture.\npub unsafe fn first() {}\npub fn second() {}\n";
    let root = repository(
        "unsafe",
        two,
        "",
        "required",
        "deny",
        "allow",
        &ratchet("rust.hygiene.unsafe"),
    );
    lock(&root);
    assert_eq!(check(&root).status, ReportStatus::Pass);

    write(&root, "src/lib.rs", one);
    let shrunk = finding(&check(&root), "RUST-HYG-005").clone();
    assert!(shrunk.message.contains("shrank"), "{}", shrunk.message);

    lock(&root);
    write(&root, "src/lib.rs", two);
    let grown = finding(&check(&root), "RUST-HYG-005").clone();
    assert!(grown.message.contains("grew"), "{}", grown.message);
    reset(&root);
}

#[test]
fn lint_measurement_distinguishes_reasoned_from_deny() {
    let source = r#"//! Fixture.
#[allow(dead_code)]
pub fn unreasoned() {}
#[allow(dead_code, reason = "legacy API")]
pub fn reasoned() {}
"#;
    let reasoned = repository(
        "lint-reasoned",
        source,
        "",
        "required",
        "allow",
        "reasoned",
        &ratchet("rust.hygiene.lint-suppressions"),
    );
    let denied = repository(
        "lint-denied",
        source,
        "",
        "required",
        "allow",
        "deny",
        &ratchet("rust.hygiene.lint-suppressions"),
    );

    assert_eq!(locked_value(&reasoned), 1);
    assert_eq!(locked_value(&denied), 2);
    reset(&reasoned);
    reset(&denied);
}

#[test]
fn baseline_discovers_every_enabled_structural_debt_kind() {
    let source = "#[allow(dead_code)]\npub unsafe fn legacy() {}\n";
    let root = repository("baseline", source, "", "required", "deny", "reasoned", "");

    let plan = discover_baseline(&root, Path::new("zrail.toml")).expect("discover baseline");
    let rules = plan
        .ratchets
        .iter()
        .map(|ratchet| ratchet.rule)
        .collect::<Vec<_>>();

    for rule in [
        "rust.module-docs",
        "rust.hygiene.unsafe",
        "rust.hygiene.lint-suppressions",
    ] {
        assert!(rules.contains(&rule), "missing {rule}: {rules:?}");
    }
    reset(&root);
}

fn repository(
    name: &str,
    source: &str,
    worker: &str,
    module_docs: &str,
    unsafe_mode: &str,
    lint_mode: &str,
    ratchets: &str,
) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!(
        "zrail-rust-debt-{name}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create source root");
    write(&root, "Cargo.toml", MANIFEST);
    write(&root, "src/lib.rs", source);
    if !worker.is_empty() {
        write(&root, "src/worker.rs", worker);
    }
    write(
        &root,
        "zrail.toml",
        &contract(module_docs, unsafe_mode, lint_mode, ratchets),
    );
    root
}

fn contract(module_docs: &str, unsafe_mode: &str, lint_mode: &str, ratchets: &str) -> String {
    format!(
        r#"schema = 1
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
module_docs = "{module_docs}"
facades = "allow"
tests = "allow"
[source.rust.macros]
mode = "allow"
[source.rust.hygiene]
unsafe = "{unsafe_mode}"
lint_suppressions = "{lint_mode}"
deny_methods = []
deny_macros = []
{ratchets}"#
    )
}

fn ratchet(rule: &str) -> String {
    format!(
        "[[ratchet]]\nrule = \"{rule}\"\ntarget = \"src/lib.rs\"\nreason = \"Legacy debt must only shrink.\"\n"
    )
}

fn locked_value(root: &Path) -> usize {
    build_lock(root, "zrail.toml".as_ref())
        .expect("build lock")
        .ratchets
        .first()
        .expect("locked ratchet")
        .value
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

fn write(root: &Path, path: &str, contents: &str) {
    fs::write(root.join(path), contents).expect("write fixture file");
}

fn reset(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).expect("reset fixture");
    }
}

const MANIFEST: &str = "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n";
