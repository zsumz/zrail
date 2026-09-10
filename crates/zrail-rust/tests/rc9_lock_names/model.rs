//! Name indexing and native graph validation preserve their separate failure stages.

pub(super) use super::super::super::model as lock_model;
use super::original;
use crate::{GovernedLockPackage, cargo::ResolvedCargoGraph};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};
use zrail_core::sha256_hex;

pub(super) use lock_model::project;
pub(super) const POLICY: &str = "docs/rc9/policies/kafka-driver.lock-packages.fragment.toml";
pub(super) const ORIGINS: &str = "crates/zrail-testkit/tests/fixtures/rc9/lock-names-origins.json";
pub(super) const CASES: &str = "crates/zrail-testkit/tests/fixtures/rc9/lock-names-cases.json";
pub(super) const MUTATIONS: &str =
    "crates/zrail-testkit/tests/fixtures/rc9/lock-names-mutations.json";
pub(super) const SOURCE_SHA: &str =
    "490a52269cd18516e625db062b0578c1e1f8c7208d57e7f9228d33f19de43573";

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
    original_count: Option<usize>,
    original_error: Option<String>,
    native_count: Option<usize>,
    native_error: Option<String>,
    stage: String,
}
#[derive(Serialize)]
pub(super) struct Row {
    case: String,
    source_sha256: String,
    original_count: Option<usize>,
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
        fs::read_to_string(project().join("crates/zrail-rust/tests/rc9_lock_names/original.rs"))
            .expect("exact excerpt");
    let start = source
        .find("    let matches = packages")
        .expect("selection");
    let excerpt = source[start..]
        .split_inclusive('\n')
        .take(4)
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
                "zrail-lock-names-{}-{:?}",
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
    assert_eq!(matrix.cases.len(), 32);
    let fixture = Fixture::new();
    let (rules, _) = lock_model::policies();
    matrix
        .cases
        .into_iter()
        .map(|case| {
            let value: toml::Value = case.source.parse().expect("well-formed TOML");
            let packages = value["package"]
                .as_array()
                .expect("array precondition held");
            let original =
                std::panic::catch_unwind(|| original::selected_count(packages, "kafka-wire"))
                    .map_err(|payload| panic_text(payload.as_ref()));
            assert_eq!(
                original.as_ref().ok().copied(),
                case.original_count,
                "{} original count",
                case.name
            );
            assert_eq!(
                original.as_ref().err(),
                case.original_error.as_ref(),
                "{} original error",
                case.name
            );
            fs::write(fixture.0.join("Cargo.lock"), &case.source).expect("fixture lock");
            let native =
                ResolvedCargoGraph::load(&fixture.0, &[]).map_err(|error| error.to_string());
            let count = native.as_ref().ok().and_then(Option::as_ref).map(|graph| {
                assert_eq!(graph.lock_sha256(), sha256_hex(case.source.as_bytes()));
                crate::lock_packages::analyze(Some(graph), &rules)
                    .expect("complete policies")
                    .into_iter()
                    .find(|row| row.policy.package == "kafka-wire")
                    .expect("wire policy")
                    .observed_count
            });
            assert_eq!(count, case.native_count, "{} native count", case.name);
            assert_eq!(
                native.as_ref().err(),
                case.native_error.as_ref(),
                "{} native stage",
                case.name
            );
            Row {
                case: case.name,
                source_sha256: sha256_hex(case.source.as_bytes()),
                original_count: original.as_ref().ok().copied(),
                original_error: original.err(),
                native_count: count,
                native_error: native.err(),
                stage: case.stage,
            }
        })
        .collect()
}

pub(super) fn original_check(root: &Path) -> Result<(), String> {
    std::panic::catch_unwind(|| super::super::check(root))
        .map_err(|payload| panic_text(payload.as_ref()))
}

pub(super) fn frozen(root: &Path) -> Vec<GovernedLockPackage> {
    original_check(root).expect("complete original caller");
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
