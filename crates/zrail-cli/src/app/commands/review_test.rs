//! Protected review treats a proposed repository only as analyzable input data.

use std::fs;

use zrail_core::{DiffReport, DiffSummary};
use zrail_rust::build_lock;

use super::{
    super::{
        git_base::git_available,
        review_fixture::{CONTRACT, fixture, fixture_with_contract, options, reset},
    },
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
fn unsupported_proposed_schema_fails_for_lock_optional_contract() {
    if !git_available() {
        return;
    }
    let contract = CONTRACT.replace("mode = \"locked\"", "mode = \"observed\"");
    let fixture = fixture_with_contract("optional-future-schema", &contract);
    let lock_path = fixture.proposal.join("zrail.lock");
    let mut lock = zrail_core::LockFile::read(&lock_path).expect("read proposed lock");
    lock.schema = zrail_core::LOCK_SCHEMA + 1;
    lock.write(&lock_path).expect("write future proposed lock");

    let result = review(&options(&fixture)).expect("review unsupported proposed lock");

    assert_eq!(result.exit_code, 1);
    assert!(result.text.contains("error[REVIEW-003]"));
    assert!(result.text.contains("latest supported schema"));
    reset(&fixture);
}
