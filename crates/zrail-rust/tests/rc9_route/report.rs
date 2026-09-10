//! Typed trusted evidence uses the existing reviewed serialization boundary.

use std::{collections::BTreeMap, path::PathBuf};

use serde::Serialize;
use serde_json::Value;

#[derive(Serialize)]
pub(super) struct Input {
    pub(super) sha256: String,
    pub(super) bytes: usize,
}

#[derive(Serialize)]
pub(super) struct Outcome {
    pub(super) case: String,
    pub(super) inputs: BTreeMap<String, Value>,
    pub(super) extra_inputs: BTreeMap<String, Value>,
    pub(super) legacy_accepted: bool,
    pub(super) native_accepted: bool,
    pub(super) observations: Vec<crate::GovernedRepositoryFile>,
    pub(super) findings: Vec<(String, String)>,
    pub(super) error: Option<String>,
}

#[derive(Serialize)]
pub(super) struct Execution {
    pub(super) test: &'static str,
    pub(super) passed: usize,
    pub(super) failed: usize,
    pub(super) ignored: usize,
    pub(super) filtered: usize,
    pub(super) binary_sha256: String,
    pub(super) list_sha256: String,
}

#[derive(Serialize)]
pub(super) struct Compilation {
    pub(super) executable: PathBuf,
    pub(super) executable_sha256: String,
    pub(super) arguments: [&'static str; 7],
    pub(super) exit_code: Option<i32>,
    pub(super) stderr_sha256: String,
    pub(super) errors: Vec<Value>,
    pub(super) execution: Option<Execution>,
}

#[derive(Serialize)]
pub(super) struct Report {
    pub(super) schema: usize,
    pub(super) implementation_commit: String,
    pub(super) implementation_tree: String,
    pub(super) snapshot: Value,
    pub(super) full_repository_qualified: bool,
    pub(super) rustc_version: String,
    pub(super) test_binary_sha256: String,
    pub(super) cargo_lock_sha256: String,
    pub(super) fixture_origins_sha256: String,
    pub(super) policy_sha256: String,
    pub(super) source_test_sha256: &'static str,
    pub(super) harness_sha256: String,
    pub(super) input_hashes: BTreeMap<String, Value>,
    pub(super) fixture_root: PathBuf,
    pub(super) fixtures: Vec<Value>,
    pub(super) limitations: [&'static str; 3],
}

pub(super) fn input(bytes: &[u8]) -> Value {
    serde_json::to_value(Input {
        sha256: zrail_core::sha256_hex(bytes),
        bytes: bytes.len(),
    })
    .expect("typed input identity")
}
