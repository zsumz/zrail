//! Generic source-operation owners enforce exact subjects and written method names.

use std::{fs, path::PathBuf};

use zrail_core::{AnalysisQuality, Finding, Report};
use zrail_rust::check_repository;

#[test]
fn operation_owners_reject_construction_method_and_field_trespassers() {
    let root = repository();
    let report = check(&root);

    for rule in [
        "state-construction",
        "transition-name",
        "epoch-read",
        "epoch-write",
        "epoch-borrow",
        "epoch-authority",
    ] {
        let finding = find(&report, rule, "OWN-003", "src/trespasser.rs");
        assert_eq!(
            finding.analysis,
            AnalysisQuality::Exact,
            "{}",
            report.human()
        );
    }
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule == "local-construction"),
        "local construction did not canonicalize: {}",
        report.human(),
    );
    assert_eq!(trespasses(&report, "epoch-read"), 1, "{}", report.human());
    assert_eq!(
        trespasses(&report, "epoch-authority"),
        3,
        "{}",
        report.human()
    );
    assert!(
        !report.findings.iter().any(|finding| {
            finding.rule == "guarded-name"
                && finding.id == "OWN-003"
                && finding.path.as_deref() == Some("src/trespasser.rs")
        }),
        "test-only method leaked into production policy: {}",
        report.human(),
    );
    reset(&root);
}

#[test]
fn exact_field_owners_fail_closed_for_unknown_receivers() {
    let root = repository();
    let report = check(&root);

    let disallowed = find(&report, "epoch-write", "OWN-003", "src/unresolved.rs");
    assert_eq!(disallowed.analysis, AnalysisQuality::Unresolved);
    let allowed = find(&report, "token-authority", "OWN-006", "src/unresolved.rs");
    assert_eq!(allowed.analysis, AnalysisQuality::Unresolved);
    assert!(allowed.message.contains("one exact Rust identity"));
    reset(&root);
}

fn repository() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "zrail-operation-ownership-{}-{:?}",
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
    write(&root, "src/unresolved.rs", UNRESOLVED);
    root
}

fn check(root: &std::path::Path) -> Report {
    check_repository(root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check operation fixture")
        .report
}

fn find<'a>(report: &'a Report, rule: &str, id: &str, path: &str) -> &'a Finding {
    report
        .findings
        .iter()
        .find(|finding| {
            finding.rule == rule && finding.id == id && finding.path.as_deref() == Some(path)
        })
        .unwrap_or_else(|| panic!("missing {rule} {id} at {path}: {}", report.human()))
}

fn trespasses(report: &Report, rule: &str) -> usize {
    report
        .findings
        .iter()
        .filter(|finding| {
            finding.rule == rule
                && finding.id == "OWN-003"
                && finding.path.as_deref() == Some("src/trespasser.rs")
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
name = "operation-fixture"
version = "0.0.0"
edition = "2024"
"#;

const LIBRARY: &str = r#"//! Operation fixture.
pub struct State { pub epoch: usize }
#[path = "owner.rs"] mod owner;
#[path = "trespasser.rs"] mod trespasser;
#[path = "unresolved.rs"] mod unresolved;
"#;

const OWNER: &str = r"//! Declared operation owner.
use crate::State;
struct Local { value: usize }
impl State {
    fn owner(&mut self) {
        let _ = self.epoch;
        self.epoch = 1;
        let _ = &mut self.epoch;
        self.transition();
        self.guarded();
    }
    fn transition(&mut self) {}
    fn guarded(&mut self) {}
}
fn construct() -> State { State { epoch: 0 } }
fn construct_local() -> Local { Local { value: 0 } }
";

const TRESPASSER: &str = r"//! Operation trespasser.
use crate::State;
impl State {
    fn trespass(&mut self) {
        let _ = self.epoch;
        self.epoch += 1;
        let _ = std::mem::replace(&mut self.epoch, 2);
        self.transition();
        #[cfg(test)] { self.guarded(); }
    }
}
fn construct() -> State { State { epoch: 0 } }
";

const UNRESOLVED: &str = r"//! Unknown receiver operations.
fn write(value: &mut Unknown) {
    let _ = value.token;
    value.epoch = 1;
    value.token = 2;
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
name = "state-construction"
kind = "type-construction"
within = ["src/**"]
match = "crate::State"
allow = ["src/owner.rs"]
reason = "Construction stays centralized."

[[owner]]
name = "local-construction"
kind = "type-construction"
within = ["src/**"]
match = "crate::owner::Local"
allow = ["src/owner.rs"]
reason = "Local construction uses its compilation-path identity."

[[owner]]
name = "transition-name"
kind = "method-name"
within = ["src/**"]
match = "transition"
allow = ["src/owner.rs"]
reason = "Written transition calls stay centralized."

[[owner]]
name = "epoch-read"
kind = "field-read"
within = ["src/**"]
match = "crate::State::epoch"
allow = ["src/owner.rs"]
reason = "Epoch reads stay centralized."

[[owner]]
name = "epoch-write"
kind = "field-write"
within = ["src/**"]
match = "crate::State::epoch"
allow = ["src/owner.rs"]
reason = "Epoch writes stay centralized."

[[owner]]
name = "epoch-borrow"
kind = "field-mutable-borrow"
within = ["src/**"]
match = "crate::State::epoch"
allow = ["src/owner.rs"]
reason = "Epoch mutable borrows stay centralized."

[[owner]]
name = "epoch-authority"
kind = "field-authority"
within = ["src/**"]
match = "crate::State::epoch"
allow = ["src/owner.rs"]
reason = "All epoch access stays centralized."

[[owner]]
name = "token-authority"
kind = "field-authority"
within = ["src/**"]
match = "crate::Other::token"
allow = ["src/unresolved.rs"]
reason = "Unknown receivers must not invent exact identities."

[[owner]]
name = "guarded-name"
kind = "method-name"
reachability = "production"
within = ["src/**"]
match = "guarded"
allow = ["src/owner.rs"]
reason = "Production authority ignores test-only calls."
"#;
