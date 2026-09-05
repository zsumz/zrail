//! Fixture evaluation preserves archived locks and reviews positive epoch migrations in memory.

use std::path::{Path, PathBuf};
use zrail_rust::check_repository;

pub(super) fn assert_rule(name: &str, rule: &str) {
    let report = check(name);
    assert!(
        report.findings.iter().any(|finding| finding.id == rule),
        "fixture {name} did not produce {rule}: {}",
        report.human()
    );
}

pub(super) fn check(name: &str) -> zrail_core::Report {
    let root = fixture_root(name);
    if matches!(name, "evidence_good" | "generated_source") {
        return check_preserved_epoch_six_fixture(&root);
    }
    check_repository(&root, Path::new("zrail.toml"), Path::new("zrail.lock"))
        .unwrap_or_else(|error| panic!("check {}: {error}", root.display()))
        .report
}

fn check_preserved_epoch_six_fixture(root: &Path) -> zrail_core::Report {
    let old = zrail_core::LockFile::read(&root.join("zrail.lock"))
        .expect("read archived rc8 fixture lock");
    assert_eq!(old.semantics, 6, "the archived fixture remains immutable");
    let candidate =
        zrail_rust::build_lock(root, Path::new("zrail.toml")).expect("observe complete fixture");
    let migration =
        zrail_core::compare_lock_epochs(&old, &candidate).expect("review fixture epoch migration");
    assert!(migration.entries.iter().all(|entry|
        entry.classification == zrail_core::LockMigrationClassification::Preserved),
        "fixture authority changed: {migration:?}");
    let mut reviewed_fixture = old;
    reviewed_fixture.semantics = zrail_core::LOCK_SEMANTICS;
    reviewed_fixture
        .analysis
        .as_mut()
        .expect("complete fixture certificate")
        .analyzer_semantics = zrail_core::LOCK_SEMANTICS;
    zrail_rust::check_repository_with_lock(root, Path::new("zrail.toml"), &reviewed_fixture)
        .expect("check migrated fixture without writing its lock")
        .report
}

fn fixture_root(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}
