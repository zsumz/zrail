//! Field owners close condition, pattern, assignee, and raw-address syntax paths.

use std::{fs, path::PathBuf};

use zrail_core::{AnalysisQuality, Report};
use zrail_rust::check_repository;

#[test]
fn let_chain_shadow_cannot_borrow_outer_exact_identity() {
    let root = repository();
    assert_owned(
        &check(&root),
        "b-secret-read",
        "src/let_chain.rs",
        AnalysisQuality::Unresolved,
    );
    reset(&root);
}

#[test]
fn match_guard_let_binding_reaches_later_conditions_and_body() {
    let root = repository();
    assert_owned(
        &check(&root),
        "b-secret-read",
        "src/match_guard.rs",
        AnalysisQuality::Unresolved,
    );
    reset(&root);
}

#[test]
fn ref_mut_pattern_is_mutation_authority() {
    let root = repository();
    assert_owned(
        &check(&root),
        "state-borrow",
        "src/pattern.rs",
        AnalysisQuality::Exact,
    );
    reset(&root);
}

#[test]
fn struct_destructuring_assignment_emits_field_writes() {
    let root = repository();
    assert_owned(
        &check(&root),
        "state-write",
        "src/destructure.rs",
        AnalysisQuality::Exact,
    );
    reset(&root);
}

#[test]
fn raw_mut_address_is_mutation_authority() {
    let root = repository();
    assert_owned(
        &check(&root),
        "state-borrow",
        "src/raw_address.rs",
        AnalysisQuality::Exact,
    );
    reset(&root);
}

fn repository() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "zrail-remaining-field-authority-{}-{:?}",
        std::process::id(),
        std::thread::current().id(),
    ));
    reset(&root);
    fs::create_dir_all(root.join("src")).expect("create fixture source");
    for (path, contents) in [
        ("Cargo.toml", MANIFEST),
        ("zrail.toml", CONTRACT),
        ("src/lib.rs", LIBRARY),
        ("src/owner.rs", OWNER),
        ("src/let_chain.rs", LET_CHAIN),
        ("src/match_guard.rs", MATCH_GUARD),
        ("src/pattern.rs", PATTERN),
        ("src/destructure.rs", DESTRUCTURE),
        ("src/raw_address.rs", RAW_ADDRESS),
    ] {
        fs::write(root.join(path), contents).expect("write fixture");
    }
    root
}

fn check(root: &std::path::Path) -> Report {
    check_repository(root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check remaining authority fixture")
        .report
}

fn assert_owned(report: &Report, rule: &str, path: &str, quality: AnalysisQuality) {
    let finding = report
        .findings
        .iter()
        .find(|finding| {
            finding.id == "OWN-003" && finding.rule == rule && finding.path.as_deref() == Some(path)
        })
        .unwrap_or_else(|| {
            panic!(
                "missing {rule} owner violation at {path}: {}",
                report.human()
            )
        });
    assert_eq!(finding.analysis, quality, "{}", report.human());
}

fn reset(root: &PathBuf) {
    if root.exists() {
        fs::remove_dir_all(root).expect("reset fixture");
    }
}

const MANIFEST: &str = r#"[package]
name = "remaining-field-authority-fixture"
version = "0.0.0"
edition = "2024"
"#;

const LIBRARY: &str = r#"//! Remaining field authority fixture.
pub struct A { pub secret: usize }
pub struct B { pub secret: usize }
pub struct State { pub epoch: usize }
pub struct Pair { pub left: usize, pub right: usize }
#[path = "owner.rs"] mod owner;
#[path = "let_chain.rs"] mod let_chain;
#[path = "match_guard.rs"] mod match_guard;
#[path = "pattern.rs"] mod pattern;
#[path = "destructure.rs"] mod destructure;
#[path = "raw_address.rs"] mod raw_address;
"#;

const OWNER: &str = r"//! Declared field authority owner.
use crate::{B, State};
fn read(value: B) { let _ = value.secret; }
fn borrow(value: &mut State) { let _ = &mut value.epoch; }
fn write(value: &mut State) { value.epoch = 1; }
";

const LET_CHAIN: &str = r"//! Let-chain shadow trespasser.
use crate::{A, B};
fn candidate() -> Option<B> { None }
pub fn trespass(outer: A) {
    if let Some(outer) = candidate()
        && outer.secret > 0
    {
        let _ = outer.secret;
    }
}
";

const MATCH_GUARD: &str = r"//! Match-guard shadow trespasser.
use crate::{A, B};
fn transform(_: &B) -> Option<B> { None }
pub fn trespass(outer: A, candidate: Option<B>) {
    match candidate {
        Some(value) if let Some(outer) = transform(&value)
            && outer.secret > 0 => { let _ = outer.secret; }
        _ => { let _ = outer; }
    }
}
";

const PATTERN: &str = r"//! Mutable pattern trespasser.
use crate::State;
pub fn trespass(state: State) {
    let State { epoch: ref mut slot } = state;
    *slot += 1;
}
";

const DESTRUCTURE: &str = r"//! Destructuring assignment trespasser.
use crate::{Pair, State};
pub fn trespass(state: &mut State, pair: Pair) {
    Pair { left: state.epoch, right: _ } = pair;
}
";

const RAW_ADDRESS: &str = r"//! Raw mutable address trespasser.
use crate::State;
pub fn trespass(state: &mut State) {
    let _pointer = &raw mut state.epoch;
}
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
name = "b-secret-read"
kind = "field-read"
within = ["src/**"]
match = "crate::B::secret"
allow = ["src/owner.rs"]
reason = "Only the declared owner reads B secrets."

[[owner]]
name = "state-borrow"
kind = "field-mutable-borrow"
within = ["src/**"]
match = "crate::State::epoch"
allow = ["src/owner.rs"]
reason = "Only the declared owner borrows the epoch mutably."

[[owner]]
name = "state-write"
kind = "field-write"
within = ["src/**"]
match = "crate::State::epoch"
allow = ["src/owner.rs"]
reason = "Only the declared owner writes the epoch."
"#;
