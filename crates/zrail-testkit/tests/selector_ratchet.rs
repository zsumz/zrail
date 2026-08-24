//! Denied methods and macros carry independent, exact tightening ratchets.

use std::{fs, path::Path};

use zrail_core::{LockedRatchet, ReportStatus};
use zrail_rust::{build_lock, check_repository, discover_baseline};

#[test]
fn selector_ratchets_measure_and_tighten_independently() {
    let root = repository("checked", SOURCE, &ratchets());
    let lock = build_lock(&root, Path::new("zrail.toml")).expect("build selector lock");

    assert_eq!(
        locked_value(&lock.ratchets, "rust.hygiene.denied-method", "unwrap"),
        1
    );
    assert_eq!(
        locked_value(&lock.ratchets, "rust.hygiene.denied-method", "expect"),
        1
    );
    assert_eq!(
        locked_value(&lock.ratchets, "rust.hygiene.denied-macro", "panic"),
        2
    );
    assert_eq!(
        locked_value(&lock.ratchets, "rust.hygiene.denied-macro", "todo"),
        1
    );
    lock.write(&root.join("zrail.lock")).expect("write lock");
    assert_eq!(check(&root).status, ReportStatus::Pass);

    write(&root, "src/lib.rs", GROWN_METHOD);
    let grown = check(&root);
    assert!(has_message(&grown, "RUST-HYG-007", "unwrap", "grew"));

    write(&root, "src/lib.rs", SHRUNK_MACRO);
    let shrunk = check(&root);
    assert!(has_message(&shrunk, "RUST-HYG-008", "panic", "shrank"));
    reset(&root);
}

#[test]
fn baseline_discovers_one_ratchet_per_denied_selector_and_file() {
    let root = repository("baseline", SOURCE, "");

    let plan = discover_baseline(&root, Path::new("zrail.toml")).expect("discover debt");
    let selectors = plan
        .ratchets
        .iter()
        .filter_map(|ratchet| {
            ratchet
                .selector
                .as_ref()
                .map(|selector| (ratchet.rule, selector.as_str(), ratchet.target.as_str()))
        })
        .collect::<Vec<_>>();

    for identity in [
        ("rust.hygiene.denied-method", "unwrap", "src/lib.rs"),
        ("rust.hygiene.denied-method", "expect", "src/lib.rs"),
        ("rust.hygiene.denied-macro", "panic", "src/lib.rs"),
        ("rust.hygiene.denied-macro", "todo", "src/lib.rs"),
    ] {
        assert!(
            selectors.contains(&identity),
            "missing {identity:?}: {selectors:?}"
        );
    }
    reset(&root);
}

fn locked_value(ratchets: &[LockedRatchet], rule: &str, selector: &str) -> usize {
    ratchets
        .iter()
        .find(|ratchet| ratchet.rule == rule && ratchet.selector.as_deref() == Some(selector))
        .unwrap_or_else(|| panic!("missing {rule}[{selector}]"))
        .value
}

fn has_message(report: &zrail_core::Report, id: &str, selector: &str, change: &str) -> bool {
    report.findings.iter().any(|finding| {
        finding.id == id && finding.message.contains(selector) && finding.message.contains(change)
    })
}

fn repository(name: &str, source: &str, ratchets: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!(
        "zrail-selector-ratchet-{name}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create source root");
    write(&root, "Cargo.toml", MANIFEST);
    write(&root, "src/lib.rs", source);
    write(&root, "zrail.toml", &contract(ratchets));
    root
}

fn contract(ratchets: &str) -> String {
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
module_docs = "required"
facades = "allow"
tests = "allow"
[source.rust.macros]
mode = "allow"
[source.rust.hygiene]
unsafe = "deny"
lint_suppressions = "allow"
deny_methods = ["unwrap", "expect"]
deny_macros = ["panic", "todo"]
{ratchets}"#
    )
}

fn ratchets() -> String {
    [
        ("rust.hygiene.denied-method", "unwrap"),
        ("rust.hygiene.denied-method", "expect"),
        ("rust.hygiene.denied-macro", "panic"),
        ("rust.hygiene.denied-macro", "todo"),
    ]
    .into_iter()
    .map(|(rule, selector)| {
        format!(
            "[[ratchet]]\nrule = \"{rule}\"\nselector = \"{selector}\"\ntarget = \"src/lib.rs\"\nreason = \"Legacy use must only shrink.\"\n"
        )
    })
    .collect()
}

fn check(root: &Path) -> zrail_core::Report {
    check_repository(root, Path::new("zrail.toml"), Path::new("zrail.lock"))
        .expect("check selector ratchets")
        .report
}

fn write(root: &Path, path: &str, contents: &str) {
    fs::write(root.join(path), contents).expect("write fixture");
}

fn reset(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).expect("reset fixture");
    }
}

const MANIFEST: &str = "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n";

const SOURCE: &str = r#"//! Fixture.
pub fn methods(first: Option<u8>, second: Option<u8>) {
    let _ = first.unwrap();
    let _ = second.expect("value");
}
pub fn macros(flag: bool) {
    if flag { panic!("first"); }
    if !flag { panic!("second"); }
    todo!();
}
"#;

const GROWN_METHOD: &str = r#"//! Fixture.
pub fn methods(first: Option<u8>, second: Option<u8>) {
    let _ = first.unwrap();
    let _ = second.unwrap();
    let _ = second.expect("value");
}
pub fn macros(flag: bool) {
    if flag { panic!("first"); }
    if !flag { panic!("second"); }
    todo!();
}
"#;

const SHRUNK_MACRO: &str = r#"//! Fixture.
pub fn methods(first: Option<u8>, second: Option<u8>) {
    let _ = first.unwrap();
    let _ = second.expect("value");
}
pub fn macros() {
    panic!("one");
    todo!();
}
"#;
