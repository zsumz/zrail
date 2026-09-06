//! Bound metadata inputs and per-policy outcomes carry no full repository qualification claim.

use std::{collections::BTreeMap, fs, path::Path};

use serde::{Deserialize, Serialize};
use zrail_core::{RepositoryFileRule, normalize_relative, sha256_hex};

use super::Suite;
use crate::repository_files::GovernedRepositoryFile;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fragment {
    repository: Rules,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Rules {
    files: Vec<RepositoryFileRule>,
}

pub(in super::super) fn policies(
    project: &Path,
    suite: &Suite,
) -> (Vec<RepositoryFileRule>, String) {
    let bytes = fs::read(project.join(format!(
        "docs/rc9/policies/kafka-driver.{}.fragment.toml",
        suite.name
    )))
    .expect("metadata policy");
    let fragment: Fragment =
        toml::from_str(std::str::from_utf8(&bytes).expect("UTF-8 policy")).expect("typed policy");
    assert_eq!(fragment.repository.files.len(), suite.policy_count);
    (fragment.repository.files, sha256_hex(&bytes))
}

pub(in super::super) fn inputs(
    root: &Path,
    policies: &[RepositoryFileRule],
    suite: &Suite,
) -> BTreeMap<String, Vec<u8>> {
    policies
        .iter()
        .flat_map(|rule| rule.include.iter().map(String::as_str))
        .chain(suite.extra_inputs.iter().copied())
        .map(|path| {
            assert_eq!(
                normalize_relative(Path::new(path)).expect("contained literal"),
                path
            );
            assert!(!path.contains(['*', '?']));
            (
                path.to_owned(),
                fs::read(root.join(path)).expect("frozen metadata input"),
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
pub(super) struct LegacyObservation {
    pub(super) policy_id: String,
    pub(super) path: String,
    pub(super) accepted: bool,
}

#[derive(Serialize)]
pub(in super::super) struct FixtureOutcome {
    pub(in super::super) policy_id: String,
    pub(in super::super) case: String,
    pub(in super::super) path: String,
    pub(in super::super) source_sha256: Option<String>,
    pub(in super::super) entry_kind: String,
    pub(in super::super) inputs: BTreeMap<String, String>,
    pub(in super::super) legacy_accepted: bool,
    pub(in super::super) legacy_failure: Option<String>,
    pub(in super::super) native_accepted: bool,
    pub(in super::super) diagnostic: Option<String>,
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
    pub(super) inputs: BTreeMap<String, String>,
    pub(super) policy_sha256: String,
    pub(super) fixture_root: String,
    pub(super) predicates: usize,
    pub(super) full_repository_qualified: bool,
    pub(super) observations: Vec<GovernedRepositoryFile>,
    pub(super) legacy: Vec<LegacyObservation>,
    pub(super) fixtures: Vec<FixtureOutcome>,
    pub(super) limitations: Vec<String>,
}
