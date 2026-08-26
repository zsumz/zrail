//! Versioned analysis certificates remain strict now and readable for migration.

use std::{fs, path::PathBuf};

use super::LockFile;

#[test]
fn current_semantics_require_a_complete_analysis_certificate() {
    assert!(
        LockFile::new("0".repeat(64))
            .render()
            .expect("current lock")
            .contains("feature_worlds = 0")
    );
    let mut missing = LockFile::new("0".repeat(64));
    missing.analysis = None;
    let error = missing.render().expect_err("missing certificate must fail");
    assert!(error.to_string().contains("analysis certificate"));

    let mut unresolved = LockFile::new("0".repeat(64));
    unresolved
        .analysis
        .as_mut()
        .expect("default certificate")
        .unresolved_bindings = 1;
    let error = unresolved.render().expect_err("partial analysis must fail");
    assert!(error.to_string().contains("unresolved analysis"));

    let mut missing_features = LockFile::new("0".repeat(64));
    let analysis = missing_features
        .analysis
        .as_mut()
        .expect("default certificate");
    analysis.cargo_features_sha256.clear();
    analysis.feature_worlds_sha256.clear();
    analysis.feature_worlds = None;
    let error = missing_features
        .render()
        .expect_err("current feature digests must be required");
    assert!(error.to_string().contains("Cargo feature"));
}

#[test]
fn released_schema_two_analysis_reads_without_feature_fields() {
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
    let legacy: LockFile = toml::from_str(&source).expect("parse released lock");
    let root = fixture_root("schema-two-features");
    reset(&root);
    let path = root.join("zrail.lock");
    fs::write(&path, legacy.render().expect("canonical released lock")).expect("write lock");

    let read = LockFile::read(&path).expect("read released lock");

    let analysis = read.analysis.expect("legacy analysis");
    assert!(analysis.cargo_features_sha256.is_empty());
    assert!(analysis.feature_worlds_sha256.is_empty());
    assert_eq!(analysis.feature_worlds, None);
    fs::remove_dir_all(root).expect("remove fixture");
}

fn fixture_root(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("zrail-lock-analysis-{name}-{}", std::process::id()))
}

fn reset(root: &PathBuf) {
    if root.exists() {
        fs::remove_dir_all(root).expect("reset fixture");
    }
    fs::create_dir_all(root).expect("create fixture");
}
