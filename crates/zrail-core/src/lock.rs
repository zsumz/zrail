//! Canonical dependency, generated-provenance, and tightening-ratchet state.

mod canonical;
mod dependency;
mod file;
use serde::{Deserialize, Serialize};
use std::{error::Error, fmt};

pub use dependency::LockedDependencySource;

/// Lock TOML format version supported by this crate.
pub const LOCK_SCHEMA: u64 = 1;
/// Resolved-architecture interpretation version produced by this crate.
pub const LOCK_SEMANTICS: u64 = 1;

/// Canonical, contract-bound architecture state stored in `zrail.lock`.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LockFile {
    /// Serialized lock format version.
    pub schema: u64,
    /// Version of the rules used to interpret resolved state.
    pub semantics: u64,
    /// `zrail-core` package version that produced the lock.
    pub producer: String,
    /// Lowercase SHA-256 digest of the governing contract bytes.
    pub contract_sha256: String,
    /// Observed packages and their source-aware dependencies.
    #[serde(default, rename = "package")]
    pub packages: Vec<LockedPackage>,
    /// Generated-source roots bound to their reviewed manifests.
    #[serde(default, rename = "generated", skip_serializing_if = "Vec::is_empty")]
    pub generated: Vec<LockedGeneratedSource>,
    /// Qualification entry points and content-bound inputs.
    #[serde(default, rename = "gate", skip_serializing_if = "Vec::is_empty")]
    pub gates: Vec<LockedGate>,
    /// Procedural-macro packages bound to their manifests.
    #[serde(
        default,
        rename = "macro_implementation",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub macro_implementations: Vec<LockedMacroImplementation>,
    /// Measured tightening-ratchet values.
    #[serde(default, rename = "ratchet")]
    pub ratchets: Vec<LockedRatchet>,
}

/// A generated-source root bound to a reviewed provenance manifest.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LockedGeneratedSource {
    /// Normalized repository-relative generated-source root.
    pub root: String,
    /// Lowercase SHA-256 digest of the provenance manifest.
    pub manifest_sha256: String,
}

/// A qualification gate bound to its executable bytes and declared inputs.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LockedGate {
    /// Stable gate name from the contract.
    pub name: String,
    /// Normalized repository-relative path to the gate entry point.
    pub path: String,
    /// Lowercase SHA-256 digest of the gate entry point.
    pub sha256: String,
    /// Additional files whose bytes participate in gate authority.
    #[serde(default, rename = "input", skip_serializing_if = "Vec::is_empty")]
    pub inputs: Vec<LockedGateInput>,
}

/// One additional file whose bytes are part of a qualification gate.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LockedGateInput {
    /// Normalized repository-relative input path.
    pub path: String,
    /// Lowercase SHA-256 digest of the input file.
    pub sha256: String,
}

/// A macro-providing package bound to its declaring manifest.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LockedMacroImplementation {
    /// Cargo package name that provides the macro implementation.
    pub package: String,
    /// Normalized repository-relative package directory.
    pub directory: String,
    /// Lowercase SHA-256 digest of the package manifest.
    pub manifest_sha256: String,
}

/// One observed Cargo package and its resolved direct dependencies.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LockedPackage {
    /// Cargo package name.
    pub name: String,
    /// Canonically ordered direct dependency identities.
    #[serde(default, rename = "dependency")]
    pub dependencies: Vec<LockedDependency>,
}

/// Complete resolved identity of one direct Cargo dependency.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LockedDependency {
    /// Cargo dependency key used by the owning package.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    /// Resolved Cargo package name.
    pub name: String,
    /// Effective Rust crate root, or `None` when an external root is unresolved.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crate_root: Option<String>,
    /// Cargo dependency table that declared the edge.
    pub kind: LockedDependencyKind,
    /// Whether the resolved package is inside or outside the workspace.
    pub scope: LockedDependencyScope,
    /// Cargo target selector, with `None` meaning all targets.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    /// Explicit optional-dependency state; current locks require a value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub optional: Option<bool>,
    /// Explicit default-feature state; current locks require a value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_features: Option<bool>,
    /// Canonically ordered, unique explicitly enabled features.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub features: Vec<String>,
    /// Complete source identity; current locks require a value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<LockedDependencySource>,
}

/// Cargo dependency table that introduced a locked edge.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LockedDependencyKind {
    /// Runtime or library dependency from `[dependencies]`.
    Normal,
    /// Test or example dependency from `[dev-dependencies]`.
    Development,
    /// Compile-time dependency from `[build-dependencies]`.
    Build,
}

/// Workspace relationship of a resolved dependency package.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LockedDependencyScope {
    /// Dependency resolves to a member of the observed workspace.
    Internal,
    /// Dependency resolves outside the observed workspace.
    External,
}

/// A positive measured ceiling retained for later tightening comparisons.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LockedRatchet {
    /// Stable rule identity that produced the measurement.
    pub rule: String,
    /// Stable governed target within the rule.
    pub target: String,
    /// Positive accepted ceiling for the target.
    pub value: usize,
}

/// Human-readable lock parsing, validation, serialization, or I/O failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LockError(String);

impl fmt::Display for LockError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for LockError {}

impl LockFile {
    /// Creates empty state for `contract_sha256` using current lock versions.
    ///
    /// The producer is this crate's package version. The digest is validated
    /// only when the lock is canonicalized, rendered, or written.
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

    /// Returns whether this lock uses the current interpretation version.
    pub fn has_current_semantics(&self) -> bool {
        self.semantics == LOCK_SEMANTICS
    }

    /// Returns whether this lock uses the supported TOML format version.
    pub fn has_supported_schema(&self) -> bool {
        self.schema == LOCK_SCHEMA
    }

    /// Compares resolved-state fields while ignoring `schema` and `producer`.
    ///
    /// This compares stored values directly and does not canonicalize either
    /// lock before comparison.
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
