//! Supported prior epochs classify every old and new authority subject.

use crate::{
    LOCK_SEMANTICS, LockFile, LockMigrationClassification, LockedExecutionReceipt,
    LockedGeneratedSource, LockedMacroSource, compare_lock_epochs,
};

#[test]
fn epoch_one_migration_is_scoped_per_authority_subject() {
    let digest = "0".repeat(64);
    let mut old = LockFile::new(&digest);
    old.schema = 1;
    old.semantics = 1;
    old.analysis = None;
    old.generated.push(LockedGeneratedSource {
        root: "generated".into(),
        manifest_sha256: "1".repeat(64),
    });
    let mut new = LockFile::new(digest);
    new.generated.push(LockedGeneratedSource {
        root: "generated".into(),
        manifest_sha256: "2".repeat(64),
    });
    new.execution_receipts.push(LockedExecutionReceipt {
        production: "src/state.rs".into(),
        test: "tests/state_test.rs".into(),
        name: "state_transitions".into(),
        receipt: "evidence/state-transitions.json".into(),
        sha256: "3".repeat(64),
        input_sha256: "4".repeat(64),
        producer: "test-runner 1.2.3".into(),
    });

    let report = compare_lock_epochs(&old, &new).expect("compare epoch one");

    assert!(report.entries.iter().any(|entry| {
        entry.rail == "rust.generated-provenance"
            && entry.classification == LockMigrationClassification::ChangedInterpretation
    }));
    assert!(report.entries.iter().any(|entry| {
        entry.rail == "analysis.inventory"
            && entry.classification == LockMigrationClassification::NewlyObservable
    }));
    for rail in [
        "analysis.cargo-features",
        "analysis.feature-worlds",
        "analysis.feature-world-count",
    ] {
        assert!(report.entries.iter().any(|entry| {
            entry.rail == rail
                && entry.classification == LockMigrationClassification::NewlyObservable
        }));
    }
    assert!(report.entries.iter().any(|entry| {
        entry.rail == "rust.test-mirror-receipt-lock"
            && entry.classification == LockMigrationClassification::NewlyObservable
    }));
    assert!(report.summary.preserved > 0);
    assert_eq!(report.summary.changed_interpretation, 1);
    assert_eq!(report.from_semantics, 1);
    assert_eq!(report.to_semantics, LOCK_SEMANTICS);
}

#[test]
fn migration_accepts_each_released_prior_epoch() {
    for (schema, semantics) in [(1, 1), (1, 2), (2, 3), (3, 4), (3, 5), (3, 6)] {
        let mut old = LockFile::new("0".repeat(64));
        old.schema = schema;
        old.semantics = semantics;
        old.analysis = None;
        let report = compare_lock_epochs(&old, &LockFile::new("0".repeat(64)))
            .expect("compare supported prior epoch");
        assert_eq!(report.from_semantics, semantics);
    }
}

#[test]
fn epoch_five_reports_exact_retired_and_new_macro_authorities() {
    let digest = "0".repeat(64);
    let mut old = LockFile::new(&digest);
    old.semantics = 5;
    old.analysis
        .as_mut()
        .expect("analysis certificate")
        .analyzer_semantics = 5;
    old.macro_sources
        .push(macro_source("macro-review", "alpha", "1.0.0"));
    let mut new = LockFile::new(digest);
    new.macro_sources
        .push(macro_source("macro-review", "beta", "2.0.0"));

    let report = compare_lock_epochs(&old, &new).expect("compare epoch five");

    let retired = report
        .entries
        .iter()
        .find(|entry| entry.rail == "rust.macro-source" && entry.subject.contains("alpha"))
        .expect("retired allowance binding");
    assert_eq!(retired.classification, LockMigrationClassification::Retired);
    let added = report
        .entries
        .iter()
        .find(|entry| entry.rail == "rust.macro-source" && entry.subject.contains("beta"))
        .expect("new allowance binding");
    assert_eq!(
        added.classification,
        LockMigrationClassification::NewlyObservable
    );
}

#[test]
fn epoch_six_preserves_one_same_name_source_and_reports_the_other() {
    let digest = "0".repeat(64);
    let mut old = LockFile::new(&digest);
    old.semantics = 6;
    old.analysis
        .as_mut()
        .expect("analysis certificate")
        .analyzer_semantics = 6;
    old.macro_sources
        .push(macro_source("macro-review", "alpha", "1.0.0"));
    let mut new = LockFile::new(digest);
    new.macro_sources
        .push(macro_source("macro-review", "alpha", "1.0.0"));
    new.macro_sources
        .push(macro_source("macro-review", "beta", "2.0.0"));

    let report = compare_lock_epochs(&old, &new).expect("compare epoch six");

    assert!(report.entries.iter().any(|entry| {
        entry.rail == "rust.macro-source"
            && entry.subject.contains("alpha")
            && entry.classification == LockMigrationClassification::Preserved
    }));
    assert!(report.entries.iter().any(|entry| {
        entry.rail == "rust.macro-source"
            && entry.subject.contains("beta")
            && entry.classification == LockMigrationClassification::NewlyObservable
    }));
}

fn macro_source(allowance: &str, package: &str, version: &str) -> LockedMacroSource {
    LockedMacroSource {
        allowance: allowance.into(),
        package: package.into(),
        version: version.into(),
        source: "registry+https://example.invalid/index".into(),
        checksum: Some("1".repeat(64)),
    }
}

#[test]
fn schema_two_analysis_without_feature_fields_remains_migratable() {
    let digest = "0".repeat(64);
    let source = format!(
        r#"schema = 2
semantics = 3
producer = "0.0.3-rc.4"
contract_sha256 = "{digest}"

[analysis]
inventory_sha256 = "{digest}"
exclusions_sha256 = "{digest}"
packages = 1
targets = 1
physical_rust_files = 1
base_source_contexts = 1
derived_source_contexts = 0
source_facts = 1
projection_queries = 0
projected_facts = 0
unresolved_bindings = 0
analyzer_semantics = 3
"#
    );
    let old: LockFile = toml::from_str(&source).expect("parse schema two lock");
    let analysis = old.analysis.as_ref().expect("legacy analysis");
    assert!(analysis.cargo_features_sha256.is_empty());
    assert!(analysis.feature_worlds_sha256.is_empty());
    assert_eq!(analysis.feature_worlds, None);

    let report = compare_lock_epochs(&old, &LockFile::new(digest)).expect("migrate schema two");
    assert!(report.entries.iter().any(|entry| {
        entry.rail == "analysis.feature-worlds"
            && entry.classification == LockMigrationClassification::NewlyObservable
    }));
}

#[test]
fn migration_rejects_unsupported_or_different_contract_authority() {
    let mut old = LockFile::new("0".repeat(64));
    old.schema = 1;
    old.semantics = 0;
    let new = LockFile::new("0".repeat(64));
    assert!(compare_lock_epochs(&old, &new).is_err());

    old.semantics = 1;
    old.schema = 2;
    assert!(compare_lock_epochs(&old, &new).is_err());

    old.schema = 1;
    let changed = LockFile::new("1".repeat(64));
    assert!(compare_lock_epochs(&old, &changed).is_err());
}
