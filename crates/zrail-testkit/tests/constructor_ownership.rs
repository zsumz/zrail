//! Constructor owners use resolved value-namespace identity, not naming style.

#[path = "constructor_ownership/fixture.rs"]
mod fixture;

use zrail_core::{AnalysisQuality, Report};
use zrail_rust::check_repository;

#[test]
fn lowercase_self_and_alias_constructors_raise_real_owner_findings() {
    let repository = fixture::Repository::new("constructors");
    let report = check(&repository);

    for rule in [
        "ticket-construction",
        "marker-construction",
        "ready-construction",
        "idle-construction",
        "record-construction",
        "uppercase-construction",
        "state-construction",
    ] {
        assert_finding(&report, rule, "src/trespasser.rs");
    }
    for rule in [
        "ticket-construction",
        "ready-construction",
        "uppercase-construction",
    ] {
        assert_finding(&report, rule, "src/capabilities.rs");
    }
    for rule in ["ready-construction", "idle-construction"] {
        assert_finding(&report, rule, "src/qualified.rs");
    }
    for rule in [
        "ticket-construction",
        "marker-construction",
        "ready-construction",
        "idle-construction",
        "record-construction",
        "uppercase-construction",
        "state-construction",
        "state-secret",
    ] {
        assert_finding(&report, rule, "src/self_trespass.rs");
    }
    assert!(
        report.findings.iter().any(|finding| {
            finding.id == "OWN-003"
                && finding.rule == "ready-construction"
                && finding.path.as_deref() == Some("src/trespasser.rs")
                && finding.analysis == AnalysisQuality::Conservative
        }),
        "glob constructor was not retained conservatively: {}",
        report.human()
    );
}

#[test]
fn proven_values_do_not_impersonate_constructors() {
    let repository = fixture::Repository::new("values");
    let report = check(&repository);

    assert!(
        !report.findings.iter().any(|finding| {
            matches!(finding.id.as_str(), "OWN-003" | "OWN-006")
                && finding.path.as_deref() == Some("src/values.rs")
        }),
        "value syntax reached a constructor owner: {}",
        report.human(),
    );
}

#[test]
fn split_impl_associated_values_do_not_impersonate_constructors() {
    let repository = fixture::Repository::new("associated-values");
    let report = check(&repository);

    assert!(
        !report.findings.iter().any(|finding| {
            matches!(finding.id.as_str(), "OWN-003" | "OWN-006")
                && finding.path.as_deref() == Some("src/values.rs")
        }),
        "associated value reached a constructor owner: {}",
        report.human(),
    );
}

fn check(repository: &fixture::Repository) -> Report {
    check_repository(
        repository.path(),
        "zrail.toml".as_ref(),
        "zrail.lock".as_ref(),
    )
    .expect("check constructor fixture")
    .report
}

fn assert_finding(report: &Report, rule: &str, path: &str) {
    let _ = report
        .findings
        .iter()
        .find(|finding| {
            finding.id == "OWN-003" && finding.rule == rule && finding.path.as_deref() == Some(path)
        })
        .unwrap_or_else(|| panic!("missing {rule} at {path}: {}", report.human()));
}
