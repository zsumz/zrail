//! Typed qualification inputs and reports; no source-resolution or lock claim.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use zrail_core::{RepositoryFileRule, repository_relative, sha256_hex};

use super::super::model::GovernedRepositoryFile;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fragment {
    repository: FileRules,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FileRules {
    files: Vec<RepositoryFileRule>,
}

pub(super) fn project() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("project root")
        .to_path_buf()
}

pub(super) fn policies() -> (Vec<RepositoryFileRule>, String) {
    let bytes = fs::read(project().join("docs/rc9/policies/kafka-driver.files.fragment.toml"))
        .expect("read reviewed file-policy fragment");
    let fragment: Fragment = toml::from_str(std::str::from_utf8(&bytes).expect("UTF-8 fragment"))
        .expect("strict typed file fragment");
    assert_eq!(fragment.repository.files.len(), 39);
    (fragment.repository.files, sha256_hex(&bytes))
}

pub(super) struct Snapshot {
    pub(super) root: PathBuf,
    pub(super) pin: serde_json::Value,
    pub(super) config: toml::Value,
    pub(super) config_sha256: String,
    pub(super) sources: BTreeMap<String, String>,
}

impl Snapshot {
    pub(super) fn load(root: PathBuf) -> Self {
        let pins: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(
                project().join("crates/zrail-testkit/tests/fixtures/rc9/snapshots.json"),
            )
            .expect("snapshot pins"),
        )
        .expect("parse snapshot pins");
        let pin = pins["consumers"]
            .as_array()
            .expect("consumers")
            .iter()
            .find(|pin| pin["name"] == "kafka-driver")
            .expect("Kafka-driver pin")
            .clone();
        let bytes = fs::read(root.join("guardrails.toml")).expect("frozen registry");
        let config: toml::Value =
            toml::from_str(std::str::from_utf8(&bytes).expect("UTF-8 registry"))
                .expect("parse legacy registry");
        let roots: Vec<String> = config["paths"]["rust_roots"]
            .clone()
            .try_into()
            .expect("legacy roots");
        let paths = crate::rules::legacy_driver_paths(&root, &roots);
        let sources: BTreeMap<_, _> = paths
            .iter()
            .map(|path| {
                (
                    repository_relative(&root, path).expect("portable path"),
                    fs::read_to_string(path).expect("legacy UTF-8 read"),
                )
            })
            .collect();
        assert_eq!(sources.len(), paths.len(), "no repeated physical files");
        assert_eq!(sources.len(), 772, "exact frozen source inventory");
        Self {
            root,
            pin,
            config,
            config_sha256: sha256_hex(&bytes),
            sources,
        }
    }
}

#[derive(Serialize)]
pub(super) struct LegacyObservation {
    pub(super) policy_id: String,
    pub(super) path: String,
    pub(super) accepted: bool,
}

#[derive(Serialize)]
pub(super) struct FixtureOutcome {
    pub(super) policy_id: String,
    pub(super) path: String,
    pub(super) entry_kind: String,
    pub(super) source_sha256: Option<String>,
    pub(super) legacy_accepted: bool,
    pub(super) native_accepted: bool,
    pub(super) diagnostic: Option<String>,
}

#[derive(Serialize)]
pub(super) struct Report {
    pub(super) schema: u64,
    pub(super) implementation_commit: String,
    pub(super) rustc_version: String,
    pub(super) test_binary_sha256: String,
    pub(super) cargo_lock_sha256: String,
    pub(super) fixture_origins_sha256: String,
    pub(super) snapshot: serde_json::Value,
    pub(super) legacy_policy_sha256: String,
    pub(super) policy_sha256: String,
    pub(super) fixture_root: String,
    pub(super) predicates: usize,
    pub(super) full_repository_qualified: bool,
    pub(super) observations: Vec<GovernedRepositoryFile>,
    pub(super) legacy: Vec<LegacyObservation>,
    pub(super) fixtures: Vec<FixtureOutcome>,
    pub(super) limitations: Vec<String>,
}
