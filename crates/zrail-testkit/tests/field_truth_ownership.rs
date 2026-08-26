//! Exact field owners fail closed when Rust leaves the declaring type unresolved.

use std::{fs, path::PathBuf};

use zrail_core::Report;
use zrail_rust::check_repository;

#[test]
fn unresolved_field_bases_are_reported_instead_of_disappearing_or_misidentifying() {
    let root = repository();
    let report = check_repository(&root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check field-truth fixture")
        .report;

    assert_eq!(
        findings(&report, "OWN-006", "src/owner.rs"),
        1,
        "{}",
        report.human()
    );
    assert_eq!(
        findings(&report, "OWN-003", "src/trespasser.rs"),
        2,
        "{}",
        report.human()
    );
    reset(&root);
}

fn repository() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "zrail-field-truth-{}-{:?}",
        std::process::id(),
        std::thread::current().id(),
    ));
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create fixture source");
    write(&root, "Cargo.toml", MANIFEST);
    write(&root, "zrail.toml", CONTRACT);
    write(&root, "src/lib.rs", LIBRARY);
    write(&root, "src/owner.rs", OWNER);
    write(&root, "src/trespasser.rs", TRESPASSER);
    root
}

fn findings(report: &Report, id: &str, path: &str) -> usize {
    report
        .findings
        .iter()
        .filter(|finding| {
            finding.rule == "epoch-authority"
                && finding.id == id
                && finding.path.as_deref() == Some(path)
        })
        .count()
}

fn write(root: &std::path::Path, path: &str, contents: &str) {
    fs::write(root.join(path), contents).expect("write fixture");
}

fn reset(root: &PathBuf) {
    if root.exists() {
        fs::remove_dir_all(root).expect("reset fixture");
    }
}

const MANIFEST: &str = r#"[package]
name = "field-truth-fixture"
version = "0.0.0"
edition = "2024"
"#;

const LIBRARY: &str = r#"//! Field-truth fixture.
pub struct State { pub epoch: usize }
pub struct StatePtr(pub State);
impl core::ops::Deref for StatePtr {
    type Target = State;
    fn deref(&self) -> &State { &self.0 }
}
#[path = "owner.rs"] mod owner;
#[path = "trespasser.rs"] mod trespasser;
"#;

const OWNER: &str = r"//! Declared owner.
use crate::{State, StatePtr};
fn own(state: &mut State, states: &mut [State], ptr: StatePtr) {
    state.epoch = 1;
    states[0].epoch = 2;
    let _ = ptr.epoch;
}
";

const TRESPASSER: &str = r"//! Trespasser.
use crate::State;
fn trespass(state: State) {
    let State { epoch, .. } = state;
    factory().epoch = epoch;
}
fn factory() -> State { State { epoch: 0 } }
";

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
module_docs = "required"
facades = "allow"
tests = "allow"
[source.rust.macros]
mode = "allow"
[source.rust.hygiene]
unsafe = "deny"
lint_suppressions = "allow"

[[owner]]
name = "epoch-authority"
kind = "field-authority"
within = ["src/**"]
match = "crate::State::epoch"
allow = ["src/owner.rs"]
reason = "Epoch authority stays centralized."
"#;
