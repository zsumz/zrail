//! Protected review treats a proposed repository only as analyzable input data.

use std::{fs, path::PathBuf};

use zrail_core::{DiffReport, DiffSummary, LockFile};
use zrail_rust::build_lock;

use crate::app::{
    args::{CommonOptions, ReviewOptions},
    output::OutputFormat,
};

use super::{
    super::git_base::{commit_all, git_available},
    architecture_denied, review,
};

#[test]
fn grant_exceptions_never_allow_debt_or_unknown_comparisons() {
    let report = |grants, debt, unknown| DiffReport {
        schema: 1,
        summary: DiffSummary {
            grants,
            debt,
            unknown,
            ..DiffSummary::default()
        },
        changes: Vec::new(),
    };

    assert!(architecture_denied(&report(1, 0, 0), false));
    assert!(!architecture_denied(&report(1, 0, 0), true));
    assert!(architecture_denied(&report(0, 1, 0), true));
    assert!(architecture_denied(&report(0, 0, 1), true));
}

#[test]
fn unchanged_compliant_source_passes_protected_review() {
    if !git_available() {
        return;
    }
    let fixture = fixture("good");

    let result = review(&options(&fixture)).expect("review proposal");

    assert_eq!(result.exit_code, 0);
    assert!(result.text.contains("Status: pass"));
    reset(&fixture);
}

#[test]
fn unchanged_contract_and_lock_cannot_hide_a_source_violation() {
    if !git_available() {
        return;
    }
    let fixture = fixture("source-violation");
    fs::write(
        fixture.proposal.join("src/lib.rs"),
        "//! fixture\n\npub unsafe fn unreviewed() {}\n",
    )
    .expect("write unsafe proposal");

    let result = review(&options(&fixture)).expect("review unsafe proposal");

    assert_eq!(result.exit_code, 1);
    assert!(result.text.contains("error[RUST-HYG-004]"));
    reset(&fixture);
}

#[test]
fn proposed_policy_weakening_cannot_authorize_violating_source() {
    if !git_available() {
        return;
    }
    let fixture = fixture("weakened-checker");
    let contract_path = fixture.proposal.join("zrail.toml");
    let contract = fs::read_to_string(&contract_path).expect("read proposed contract");
    fs::write(
        &contract_path,
        contract.replace("unsafe = \"deny\"", "unsafe = \"allow\""),
    )
    .expect("weaken proposed contract");
    fs::write(
        fixture.proposal.join("src/lib.rs"),
        "//! fixture\n\npub unsafe fn unreviewed() {}\n",
    )
    .expect("write source using proposed grant");
    build_lock(&fixture.proposal, std::path::Path::new("zrail.toml"))
        .expect("forge proposal lock")
        .write(&fixture.proposal.join("zrail.lock"))
        .expect("write forged proposal lock");

    let result = review(&options(&fixture)).expect("review weakened proposal");

    assert_eq!(result.exit_code, 1);
    assert!(result.text.contains("GRANT rust.unsafe"));
    reset(&fixture);
}

#[test]
fn proposed_lock_must_match_independently_observed_source() {
    if !git_available() {
        return;
    }
    let fixture = fixture("stale-lock");
    fs::write(
        fixture.proposal.join("Cargo.toml"),
        concat!(
            "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
            "edition = \"2024\"\n\n[dependencies]\nserde = \"1\"\n",
        ),
    )
    .expect("change observed dependency graph");

    let result = review(&options(&fixture)).expect("review stale proposal lock");

    assert_eq!(result.exit_code, 1);
    assert!(result.text.contains("error[REVIEW-002]"));
    assert!(result.text.contains("error[LOCK-005]"));
    reset(&fixture);
}

#[test]
fn proposal_cargo_configuration_is_rejected_without_execution() {
    if !git_available() {
        return;
    }
    let fixture = fixture("cargo-config");
    fs::create_dir_all(fixture.proposal.join(".cargo")).expect("create Cargo configuration");
    fs::write(
        fixture.proposal.join(".cargo/config.toml"),
        "[build]\nrustc-wrapper = \"./authority-wrapper\"\n",
    )
    .expect("write hostile Cargo configuration");
    fs::write(
        fixture.proposal.join("authority-wrapper"),
        "proposal-controlled executable\n",
    )
    .expect("write inert wrapper");

    let result = review(&options(&fixture)).expect("review hostile Cargo configuration");

    assert_eq!(result.exit_code, 1);
    assert!(result.text.contains("error[CARGO-CONFIG-001]"));
    assert!(result.text.contains(".cargo/config.toml"));
    assert!(!fixture.proposal.join("wrapper-ran").exists());
    reset(&fixture);
}

#[test]
fn changed_producer_with_stable_semantics_is_reviewable() {
    if !git_available() {
        return;
    }
    let fixture = fixture("producer");
    let lock_path = fixture.proposal.join("zrail.lock");
    let mut lock = LockFile::read(&lock_path).expect("read proposed lock");
    lock.producer = "0.0.2".into();
    lock.write(&lock_path).expect("write newer producer");

    let result = review(&options(&fixture)).expect("review newer producer");

    assert_eq!(result.exit_code, 0);
    reset(&fixture);
}

#[test]
fn unsupported_proposed_schema_fails_for_lock_optional_contract() {
    if !git_available() {
        return;
    }
    let contract = CONTRACT.replace("mode = \"locked\"", "mode = \"observed\"");
    let fixture = fixture_with_contract("optional-future-schema", &contract);
    let lock_path = fixture.proposal.join("zrail.lock");
    let mut lock = LockFile::read(&lock_path).expect("read proposed lock");
    lock.schema = zrail_core::LOCK_SCHEMA + 1;
    lock.write(&lock_path).expect("write future proposed lock");

    let result = review(&options(&fixture)).expect("review unsupported proposed lock");

    assert_eq!(result.exit_code, 1);
    assert!(result.text.contains("error[REVIEW-003]"));
    assert!(result.text.contains("latest supported schema"));
    reset(&fixture);
}

fn fixture(name: &str) -> Fixture {
    fixture_with_contract(name, CONTRACT)
}

fn fixture_with_contract(name: &str, contract: &str) -> Fixture {
    let base = std::env::temp_dir().join(format!(
        "zrail-review-{name}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let fixture = Fixture {
        authority: base.join("authority"),
        proposal: base.join("proposal"),
        base,
    };
    reset(&fixture);
    write_repository(&fixture.authority, contract);
    build_lock(&fixture.authority, std::path::Path::new("zrail.toml"))
        .expect("build authority lock")
        .write(&fixture.authority.join("zrail.lock"))
        .expect("write authority lock");
    commit_all(&fixture.authority);
    copy_repository(&fixture.authority, &fixture.proposal);
    fixture
}

fn write_repository(root: &std::path::Path, contract: &str) {
    fs::create_dir_all(root.join("src")).expect("create source directory");
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
    )
    .expect("write Cargo manifest");
    fs::write(root.join("src/lib.rs"), "//! fixture\n").expect("write Rust source");
    fs::write(root.join("zrail.toml"), contract).expect("write contract");
}

fn copy_repository(source: &std::path::Path, destination: &std::path::Path) {
    fs::create_dir_all(destination.join("src")).expect("create proposal source directory");
    for path in ["Cargo.toml", "zrail.toml", "zrail.lock", "src/lib.rs"] {
        let target = destination.join(path);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).expect("create proposal parent");
        }
        fs::copy(source.join(path), target).expect("copy proposal input");
    }
}

fn options(fixture: &Fixture) -> ReviewOptions {
    ReviewOptions {
        common: CommonOptions {
            root: fixture.proposal.clone(),
            config: "zrail.toml".into(),
            lock: "zrail.lock".into(),
            format: OutputFormat::Human,
        },
        authority_root: fixture.authority.clone(),
        base: "HEAD".into(),
        allow_grants: false,
    }
}

fn reset(fixture: &Fixture) {
    if fixture.base.exists() {
        fs::remove_dir_all(&fixture.base).expect("reset fixture");
    }
}

struct Fixture {
    base: PathBuf,
    authority: PathBuf,
    proposal: PathBuf,
}

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
mode = "locked"
unassigned_packages = "allow"
cycles = "deny"

[source.rust]
module_docs = "required"
facades = "allow"
tests = "allow"

[source.rust.hygiene]
unsafe = "deny"
lint_suppressions = "allow"
deny_methods = []
deny_macros = []
"#;
