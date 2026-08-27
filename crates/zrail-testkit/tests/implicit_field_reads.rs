//! Implicit field reads reach exact owners, including through local imports.

use std::{fs, path::PathBuf};

use zrail_core::{AnalysisQuality, Finding, Report};
use zrail_rust::check_repository;

#[test]
fn destructuring_source_read_reaches_field_owner() {
    let root = repository();
    let report = check(&root);

    let finding = find(&report, "vault-secret", "src/destructure.rs");
    assert_eq!(
        finding.analysis,
        AnalysisQuality::Exact,
        "{}",
        report.human()
    );
    assert!(!report.findings.iter().any(|finding| {
        finding.rule == "vault-spare"
            && finding.id == "OWN-003"
            && finding.path.as_deref() == Some("src/destructure.rs")
    }));
    reset(&root);
}

#[test]
fn imported_local_update_reaches_each_field_owner_exactly() {
    let root = repository();
    let report = check(&root);

    for rule in ["state-secret", "state-spare"] {
        let finding = find(&report, rule, "src/update.rs");
        assert_eq!(
            finding.analysis,
            AnalysisQuality::Exact,
            "{}",
            report.human()
        );
    }
    reset(&root);
}

fn repository() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "zrail-implicit-field-reads-{}-{:?}",
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
        ("src/destructure.rs", DESTRUCTURE),
        ("src/update.rs", UPDATE),
    ] {
        fs::write(root.join(path), contents).expect("write fixture");
    }
    root
}

fn check(root: &std::path::Path) -> Report {
    check_repository(root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check implicit-field fixture")
        .report
}

fn find<'a>(report: &'a Report, rule: &str, path: &str) -> &'a Finding {
    report
        .findings
        .iter()
        .find(|finding| {
            finding.rule == rule && finding.id == "OWN-003" && finding.path.as_deref() == Some(path)
        })
        .unwrap_or_else(|| {
            panic!(
                "missing {rule} owner violation at {path}: {}",
                report.human()
            )
        })
}

fn reset(root: &PathBuf) {
    if root.exists() {
        fs::remove_dir_all(root).expect("reset fixture");
    }
}

const MANIFEST: &str = r#"[package]
name = "implicit-field-reads-fixture"
version = "0.0.0"
edition = "2024"
"#;

const LIBRARY: &str = r#"//! Implicit field-read fixture.
pub struct Vault { pub secret: usize, pub spare: usize }
pub struct State { pub public: usize, pub secret: usize, pub spare: usize }
#[path = "owner.rs"] mod owner;
#[path = "destructure.rs"] mod destructure;
#[path = "update.rs"] mod update;
"#;

const OWNER: &str = r"//! Declared field-read owners.
use crate::{State, Vault};
fn own_vault(value: Vault) { let _ = value.secret; let _ = value.spare; }
fn own_state(value: State) { let _ = value.secret; let _ = value.spare; }
";

const DESTRUCTURE: &str = r"//! Destructuring source reader.
use crate::Vault;
pub fn trespass(vault: Vault, mut sink: usize) {
    Vault { secret: sink, spare: _ } = vault;
}
";

const UPDATE: &str = r"//! Functional-update source reader.
use crate::State;
pub fn trespass(previous: State) -> State {
    State { public: 10, ..previous }
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
name = "vault-secret"
kind = "field-read"
within = ["src/**"]
match = "crate::Vault::secret"
allow = ["src/owner.rs"]
reason = "Vault secret reads stay centralized."

[[owner]]
name = "vault-spare"
kind = "field-read"
within = ["src/**"]
match = "crate::Vault::spare"
allow = ["src/owner.rs"]
reason = "Ignored assignee fields are not source reads."

[[owner]]
name = "state-secret"
kind = "field-read"
within = ["src/**"]
match = "crate::State::secret"
allow = ["src/owner.rs"]
reason = "State secret reads stay centralized."

[[owner]]
name = "state-spare"
kind = "field-read"
within = ["src/**"]
match = "crate::State::spare"
allow = ["src/owner.rs"]
reason = "State spare reads stay centralized."
"#;
