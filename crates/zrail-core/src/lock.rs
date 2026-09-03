//! Canonical dependency, generated-provenance, and tightening-ratchet state.

mod analysis;
mod canonical;
mod dependency;
mod error;
mod file;
mod item_macro;
mod macro_source;
mod receipt;
mod state;
use serde::{Deserialize, Serialize};

pub use analysis::{LockedAnalysis, LockedContractSource};
pub use dependency::LockedDependencySource;
pub use error::LockError;
pub use item_macro::LockedItemMacroManifest;
pub use macro_source::LockedMacroSource;
pub use receipt::LockedExecutionReceipt;
pub use state::{LockedGate, LockedGateInput, LockedGeneratedSource, LockedMacroImplementation};

/// Lock TOML format version supported by this crate.
pub const LOCK_SCHEMA: u64 = 3;
/// Resolved-architecture interpretation version produced by this crate.
pub const LOCK_SEMANTICS: u64 = 6;

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
    /// Complete analyzed-universe certificate; required by current semantics.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub analysis: Option<LockedAnalysis>,
    /// Observed packages and their source-aware dependencies.
    #[serde(default, rename = "package")]
    pub packages: Vec<LockedPackage>,
    /// Generated-source roots bound to their reviewed manifests.
    #[serde(default, rename = "generated", skip_serializing_if = "Vec::is_empty")]
    pub generated: Vec<LockedGeneratedSource>,
    /// Qualification entry points and content-bound inputs.
    #[serde(default, rename = "gate", skip_serializing_if = "Vec::is_empty")]
    pub gates: Vec<LockedGate>,
    /// Exact execution receipts whose bytes grant test-mirror authority.
    #[serde(
        default,
        rename = "execution_receipt",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub execution_receipts: Vec<LockedExecutionReceipt>,
    /// Repository macro packages bound to their implementation input sets.
    #[serde(
        default,
        rename = "macro_implementation",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub macro_implementations: Vec<LockedMacroImplementation>,
    /// Cargo.lock-resolved packages whose exact identity grants macro authority.
    #[serde(
        default,
        rename = "macro_source",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub macro_sources: Vec<LockedMacroSource>,
    /// Exact item-macro namespace manifests.
    #[serde(
        default,
        rename = "item_macro_manifest",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub item_macro_manifests: Vec<LockedItemMacroManifest>,
    /// Measured tightening-ratchet values.
    #[serde(default, rename = "ratchet")]
    pub ratchets: Vec<LockedRatchet>,
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
    /// Optional normalized denied-operation selector measured independently.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
    /// Stable governed target within the rule.
    pub target: String,
    /// Positive accepted ceiling for the target.
    pub value: usize,
}

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
            analysis: Some(LockedAnalysis::default()),
            packages: Vec::new(),
            generated: Vec::new(),
            gates: Vec::new(),
            execution_receipts: Vec::new(),
            macro_implementations: Vec::new(),
            macro_sources: Vec::new(),
            item_macro_manifests: Vec::new(),
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
            && match (&self.analysis, &other.analysis) {
                (Some(left), Some(right)) => left.same_authority(right),
                (None, None) => true,
                _ => false,
            }
            && self.packages == other.packages
            && self.generated == other.generated
            && self.gates == other.gates
            && self.execution_receipts == other.execution_receipts
            && self.macro_implementations == other.macro_implementations
            && self.macro_sources == other.macro_sources
            && self.item_macro_manifests == other.item_macro_manifests
            && self.ratchets == other.ratchets
    }
}

#[cfg(test)]
#[path = "lock_test.rs"]
mod lock_test;
