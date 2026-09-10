//! Original array preconditions remain distinct from later native graph validation.

pub(super) use super::super::super::model as lock_model;
use super::original;
use crate::cargo::ResolvedCargoGraph;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};
use zrail_core::sha256_hex;

pub(super) const POLICY: &str = "docs/rc9/policies/kafka-driver.lock-packages.fragment.toml";
pub(super) const ORIGINS: &str = "crates/zrail-testkit/tests/fixtures/rc9/lock-array-origins.json";
pub(super) const CASES: &str = "crates/zrail-testkit/tests/fixtures/rc9/lock-array-cases.json";
pub(super) const SOURCE_SHA: &str =
    "490a52269cd18516e625db062b0578c1e1f8c7208d57e7f9228d33f19de43573";
pub(super) use lock_model::project;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Matrix {
    schema: u32,
    cases: Vec<Case>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    name: String,
    source: String,
    array_count: Option<usize>,
    native_count: Option<usize>,
    native_error: Option<String>,
    stage: String,
}
#[derive(Serialize)]
pub(super) struct Row {
    case: String,
    source_sha256: String,
    array_count: Option<usize>,
    original_error: Option<String>,
    native_count: Option<usize>,
    native_error: Option<String>,
    stage: String,
}

pub(super) fn panic_text(payload: &(dyn std::any::Any + Send)) -> String {
    payload
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| payload.downcast_ref::<&str>().map(|text| (*text).into()))
        .expect("original panic text")
}

pub(super) fn source_binding() {
    let registry: serde_json::Value =
        serde_json::from_slice(&fs::read(project().join(ORIGINS)).expect("origins")).expect("JSON");
    let source =
        fs::read_to_string(project().join("crates/zrail-rust/tests/rc9_lock_array/original.rs"))
            .expect("original excerpt");
    let start = source
        .find("    let packages = lock[\"package\"]")
        .expect("exact precondition");
    let excerpt = source[start..]
        .split_inclusive('\n')
        .take(3)
        .collect::<String>();
    assert_eq!(
        sha256_hex(excerpt.as_bytes()),
        registry["extractions"][0]["extracted_sha256"]
            .as_str()
            .expect("excerpt digest")
    );
}

pub(super) struct Fixture(pub(super) PathBuf);
impl Fixture {
    pub(super) fn new() -> Self {
        let root = std::env::temp_dir()
            .canonicalize()
            .expect("canonical temp")
            .join(format!(
                "zrail-lock-array-{}-{:?}",
                std::process::id(),
                std::thread::current().id()
            ));
        assert!(!root.exists(), "fresh test-owned fixture");
        fs::create_dir(&root).expect("create fixture");
        Self(root)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("remove only test-owned fixture");
    }
}

pub(super) fn matrix() -> Vec<Row> {
    let matrix: Matrix = serde_json::from_slice(&fs::read(project().join(CASES)).expect("cases"))
        .expect("typed matrix");
    assert_eq!(matrix.schema, 1);
    assert_eq!(matrix.cases.len(), 24);
    let fixture = Fixture::new();
    matrix
        .cases
        .into_iter()
        .map(|case| {
            let value: toml::Value = case.source.parse().expect("well-formed TOML input");
            let original = std::panic::catch_unwind(|| original::package_count(&value))
                .map_err(|payload| panic_text(payload.as_ref()));
            assert_eq!(
                original.as_ref().ok().copied(),
                case.array_count,
                "{}: original array stage",
                case.name
            );
            fs::write(fixture.0.join("Cargo.lock"), &case.source).expect("fixture lock");
            let native =
                ResolvedCargoGraph::load(&fixture.0, &[]).map_err(|error| error.to_string());
            let count = native.as_ref().ok().and_then(Option::as_ref).map(|graph| {
                assert_eq!(graph.lock_sha256(), sha256_hex(case.source.as_bytes()));
                graph.all_packages().count()
            });
            assert_eq!(count, case.native_count, "{}: native graph", case.name);
            assert_eq!(
                native.as_ref().err(),
                case.native_error.as_ref(),
                "{}: precise failure stage",
                case.name
            );
            Row {
                case: case.name,
                source_sha256: sha256_hex(case.source.as_bytes()),
                array_count: original.as_ref().ok().copied(),
                original_error: original.err(),
                native_count: count,
                native_error: native.err(),
                stage: case.stage,
            }
        })
        .collect()
}

#[derive(Serialize)]
pub(super) struct EmptyResult {
    policy_id: String,
    original_error: String,
    native_diagnostic: String,
    observed: crate::GovernedLockPackage,
}

pub(super) fn empty_counts() -> Vec<EmptyResult> {
    let fixture = Fixture::new();
    let source = "version = 4\npackage = []\n";
    fs::write(fixture.0.join("Cargo.lock"), source).expect("empty lock");
    let value: toml::Value = source.parse().expect("TOML");
    assert_eq!(original::package_count(&value), 0);
    let graph = ResolvedCargoGraph::load(&fixture.0, &[])
        .expect("empty graph allowed before workspace mapping")
        .expect("lock present");
    let (rules, _) = lock_model::policies();
    let observed =
        crate::lock_packages::analyze(Some(&graph), &rules).expect("complete empty graph");
    observed
        .into_iter()
        .map(|row| {
            let zrail_core::LockPackageAssertion::Exact { identities } = &row.policy.assertion
            else {
                panic!("exact policy");
            };
            let identity = &identities[0];
            let failure = std::panic::catch_unwind(|| {
                super::super::assert_locked(
                    &value,
                    &row.policy.package,
                    &format!("={}", identity.version),
                    identity.checksum.as_deref().expect("checksum"),
                );
            })
            .expect_err("required count rejects empty array");
            let original_error = panic_text(failure.as_ref());
            assert!(original_error.contains("must resolve exactly once"));
            assert!(!original_error.contains("must contain package entries"));
            assert!(!row.satisfied);
            assert_eq!(row.observed_count, 0);
            let mut sink = zrail_core::FindingSink::default();
            crate::lock_packages::evaluate(std::slice::from_ref(&row), &mut sink);
            let findings = sink.into_findings();
            assert_eq!(findings.len(), 1);
            assert_eq!(findings[0].id, "DEP-LOCK-001");
            assert_eq!(findings[0].rule, row.policy_id);
            EmptyResult {
                policy_id: row.policy_id.clone(),
                original_error,
                native_diagnostic: findings[0].id.clone(),
                observed: row,
            }
        })
        .collect()
}

pub(super) fn frozen(root: &Path) -> Vec<crate::GovernedLockPackage> {
    super::super::check(root);
    let workspace = lock_model::workspace(root);
    let (rules, _) = lock_model::policies();
    let native = lock_model::native(root, &workspace, &rules).expect("complete frozen graph");
    assert_eq!(native.len(), 5);
    assert!(
        native
            .iter()
            .all(|row| row.satisfied && row.observed_count == 1)
    );
    native
}
