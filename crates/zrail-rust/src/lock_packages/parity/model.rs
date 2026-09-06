//! Trusted parity derives workspace facts with the native Cargo loader and binds every input.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use zrail_core::{LockPackageRule, load_contract, sha256_hex};

use crate::{
    cargo::{Package, ResolvedCargoGraph, load_cargo_workspace},
    inventory::inventory_repository,
    lock_packages::GovernedLockPackage,
};

pub(super) const INPUTS: &[&str] = &[
    "Cargo.toml",
    "Cargo.lock",
    "guardrails.toml",
    "crates/kafka-driver-core/Cargo.toml",
    "crates/kafka-driver-probe/Cargo.toml",
    "crates/kafka-driver-sim/Cargo.toml",
    "crates/kafka-driver-transport/Cargo.toml",
    "src/lib.rs",
    "crates/kafka-driver-core/src/lib.rs",
    "crates/kafka-driver-probe/src/main.rs",
    "crates/kafka-driver-sim/src/lib.rs",
    "crates/kafka-driver-transport/src/lib.rs",
];

pub(super) fn project() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("project root")
        .to_owned()
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fragment {
    dependencies: Rules,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Rules {
    lock_package: Vec<LockPackageRule>,
}

pub(super) fn policies() -> (Vec<LockPackageRule>, String) {
    let bytes =
        fs::read(project().join("docs/rc9/policies/kafka-driver.lock-packages.fragment.toml"))
            .expect("generated lock fragment");
    let parsed: Fragment = toml::from_str(std::str::from_utf8(&bytes).expect("UTF-8 policy"))
        .expect("typed lock rules");
    assert_eq!(parsed.dependencies.lock_package.len(), 5);
    (parsed.dependencies.lock_package, sha256_hex(&bytes))
}

pub(super) fn workspace(root: &Path) -> Vec<Package> {
    let fixture = project().join("crates/zrail-testkit/tests/fixtures/good");
    let mut contract = load_contract(&fixture, "zrail.toml".as_ref())
        .expect("native fixture contract")
        .contract;
    // This selects Cargo manifests only for this graph qualification; no source certificate is produced.
    contract.repository.roots = vec![".".into()];
    let inventory = inventory_repository(root, &contract).expect("complete physical discovery");
    let cargo = load_cargo_workspace(&inventory).expect("native Cargo workspace projection");
    assert_eq!(cargo.packages.len(), 5);
    cargo.packages
}

pub(super) fn native(
    root: &Path,
    workspace: &[Package],
    rules: &[LockPackageRule],
) -> Result<Vec<GovernedLockPackage>, String> {
    let graph = ResolvedCargoGraph::load(root, workspace).map_err(|error| error.to_string())?;
    crate::lock_packages::analyze(graph.as_ref(), rules)
}

pub(super) fn inputs(root: &Path) -> BTreeMap<String, Vec<u8>> {
    INPUTS
        .iter()
        .map(|path| {
            (
                (*path).into(),
                fs::read(root.join(path)).expect("bound frozen input"),
            )
        })
        .collect()
}

pub(super) fn hashes(inputs: &BTreeMap<String, Vec<u8>>) -> BTreeMap<String, String> {
    inputs
        .iter()
        .map(|(path, bytes)| (path.clone(), sha256_hex(bytes)))
        .collect()
}

#[derive(Serialize)]
pub(super) struct FixtureOutcome {
    pub(super) policy_id: String,
    pub(super) case: String,
    pub(super) path: String,
    pub(super) source_sha256: String,
    pub(super) inputs: BTreeMap<String, String>,
    pub(super) legacy_accepted: bool,
    pub(super) legacy_failure: Option<String>,
    pub(super) native_accepted: bool,
    pub(super) diagnostic: Option<String>,
    pub(super) observed: GovernedLockPackage,
}

#[derive(Serialize)]
pub(super) struct WorkspaceIdentity<'a> {
    pub(super) name: &'a str,
    pub(super) directory: &'a str,
}

#[derive(Serialize)]
pub(super) struct LegacyObservation {
    pub(super) policy_id: String,
    pub(super) path: &'static str,
    pub(super) accepted: bool,
}

#[derive(Serialize)]
pub(super) struct Report<'a> {
    pub(super) schema: u64,
    pub(super) implementation_commit: String,
    pub(super) implementation_tree: String,
    pub(super) snapshot: serde_json::Value,
    pub(super) policy_sha256: String,
    pub(super) inputs: BTreeMap<String, String>,
    pub(super) cargo_inventory_roots: [&'static str; 1],
    pub(super) workspace_packages: Vec<WorkspaceIdentity<'a>>,
    pub(super) projection_contract_sha256: String,
    pub(super) rustc_version: String,
    pub(super) test_binary_sha256: String,
    pub(super) cargo_lock_sha256: String,
    pub(super) fixture_origins_sha256: String,
    pub(super) fixture_root: &'a str,
    pub(super) predicates: usize,
    pub(super) full_repository_qualified: bool,
    pub(super) observations: Vec<GovernedLockPackage>,
    pub(super) legacy: Vec<LegacyObservation>,
    pub(super) fixtures: Vec<FixtureOutcome>,
    pub(super) limitations: [&'static str; 5],
}
