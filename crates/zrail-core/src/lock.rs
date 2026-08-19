//! Canonical dependency, generated-provenance, and tightening-ratchet state.

mod canonical;
mod dependency;
mod file;
use serde::{Deserialize, Serialize};
use std::{error::Error, fmt};

pub use dependency::LockedDependencySource;

pub const LOCK_SCHEMA: u64 = 1;
pub const LOCK_SEMANTICS: u64 = 1;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LockFile {
    pub schema: u64,
    pub semantics: u64,
    pub producer: String,
    pub contract_sha256: String,
    #[serde(default, rename = "package")]
    pub packages: Vec<LockedPackage>,
    #[serde(default, rename = "generated", skip_serializing_if = "Vec::is_empty")]
    pub generated: Vec<LockedGeneratedSource>,
    #[serde(default, rename = "gate", skip_serializing_if = "Vec::is_empty")]
    pub gates: Vec<LockedGate>,
    #[serde(
        default,
        rename = "macro_implementation",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub macro_implementations: Vec<LockedMacroImplementation>,
    #[serde(default, rename = "ratchet")]
    pub ratchets: Vec<LockedRatchet>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LockedGeneratedSource {
    pub root: String,
    pub manifest_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LockedGate {
    pub name: String,
    pub path: String,
    pub sha256: String,
    #[serde(default, rename = "input", skip_serializing_if = "Vec::is_empty")]
    pub inputs: Vec<LockedGateInput>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LockedGateInput {
    pub path: String,
    pub sha256: String,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LockedMacroImplementation {
    pub package: String,
    pub directory: String,
    pub manifest_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LockedPackage {
    pub name: String,
    #[serde(default, rename = "dependency")]
    pub dependencies: Vec<LockedDependency>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LockedDependency {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crate_root: Option<String>,
    pub kind: LockedDependencyKind,
    pub scope: LockedDependencyScope,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub optional: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_features: Option<bool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub features: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<LockedDependencySource>,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LockedDependencyKind {
    Normal,
    Development,
    Build,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LockedDependencyScope {
    Internal,
    External,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LockedRatchet {
    pub rule: String,
    pub target: String,
    pub value: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LockError(String);

impl fmt::Display for LockError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for LockError {}

impl LockFile {
    pub fn new(contract_sha256: impl Into<String>) -> Self {
        Self {
            schema: LOCK_SCHEMA,
            semantics: LOCK_SEMANTICS,
            producer: env!("CARGO_PKG_VERSION").into(),
            contract_sha256: contract_sha256.into(),
            packages: Vec::new(),
            generated: Vec::new(),
            gates: Vec::new(),
            macro_implementations: Vec::new(),
            ratchets: Vec::new(),
        }
    }

    pub fn has_current_semantics(&self) -> bool {
        self.semantics == LOCK_SEMANTICS
    }

    pub fn has_supported_schema(&self) -> bool {
        self.schema == LOCK_SCHEMA
    }

    pub fn same_resolved_state(&self, other: &Self) -> bool {
        self.semantics == other.semantics
            && self.contract_sha256 == other.contract_sha256
            && self.packages == other.packages
            && self.generated == other.generated
            && self.gates == other.gates
            && self.macro_implementations == other.macro_implementations
            && self.ratchets == other.ratchets
    }
}

#[cfg(test)]
#[path = "lock_test.rs"]
mod lock_test;
