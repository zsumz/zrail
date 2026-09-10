//! Explicit inspection flows through public coverage, explain, check and lock construction.

use super::support::{configured, coverage};
use std::fs;
use zrail_core::{ReportStatus, RepositoryFilePredicate};

const RULE: &str = r#"
[[repository.files]]
name = "inspection"
include = ["assets/**"]
entry = "any"
reason = "Inspect every physical asset entry, including empty trees."
predicate = { kind = "inspect" }
"#;

#[test]
fn inspection_public_reports_expose_empty_and_populated_scope_with_lock_binding() {
    let repository = configured(RULE);
    repository.lock();
    assert_eq!(repository.check().report.status, ReportStatus::Pass);
    let report = coverage(&repository);
    assert!(report.repository_files[0].satisfied);
    assert!(report.repository_files[0].entries.is_empty());
    assert_eq!(
        report.repository_files[0].claim,
        "physical-entry-inspection"
    );
    assert_eq!(
        report.repository_files[0].policy.predicate,
        RepositoryFilePredicate::Inspect {}
    );
    let hypothetical = zrail_rust::explain_hypothetical_path(
        &repository.0,
        "zrail.toml".as_ref(),
        "assets/future.bin".as_ref(),
    )
    .expect("hypothetical physical path");
    assert!(hypothetical.repository_files[0].entry.is_none());
    assert_eq!(hypothetical.repository_files[0].scope_entries, 0);
    assert!(hypothetical.repository_files[0].scope_satisfied);
    let before = repository
        .check()
        .candidate_lock
        .expect("empty selection lock");
    fs::create_dir(repository.0.join("assets")).expect("asset directory");
    fs::write(repository.0.join("assets/raw.bin"), [0xff, 0]).expect("non-UTF-8 bytes");
    let result = repository.check();
    assert!(
        result
            .report
            .findings
            .iter()
            .any(|finding| finding.id == "LOCK-028")
    );
    assert_ne!(
        before.analysis,
        result
            .candidate_lock
            .expect("complete physical observation")
            .analysis
    );
    repository.lock();
    assert_eq!(repository.check().report.status, ReportStatus::Pass);
    let explained = repository.explain("assets/raw.bin");
    let policy = &explained.repository_files[0];
    assert_eq!(policy.claim, "physical-entry-inspection");
    assert_eq!(policy.scope_entries, 2);
    let entry = policy.entry.as_ref().expect("actual selected entry");
    assert_eq!(entry.path, "assets/raw.bin");
    assert_eq!(entry.kind, "file");
    assert!(entry.sha256.is_none() && entry.bytes.is_none());
    assert!(explained.human().contains("Inspect"));
    assert_eq!(
        coverage(&repository).json().expect("JSON"),
        coverage(&repository).json().expect("repeat JSON")
    );
}

#[test]
fn inspection_incomplete_scope_produces_no_partial_public_result_or_lock() {
    let repository = configured(RULE);
    repository.lock();
    let old_lock = fs::read(repository.0.join("zrail.lock")).expect("prior authority");
    fs::create_dir_all(repository.0.join("assets/.git")).expect("pruned selected subtree");
    let config = "zrail.toml".as_ref();
    for error in [
        zrail_rust::build_lock(&repository.0, config).expect_err("no partial lock"),
        zrail_rust::governed_surface_report(&repository.0, config)
            .expect_err("no partial coverage"),
        zrail_rust::check_repository(&repository.0, config, "zrail.lock".as_ref())
            .expect_err("no unenforced check"),
    ] {
        assert!(error.to_string().contains("REP-FILE-006"), "{error}");
    }
    assert_eq!(
        fs::read(repository.0.join("zrail.lock")).expect("unchanged authority"),
        old_lock
    );
}
