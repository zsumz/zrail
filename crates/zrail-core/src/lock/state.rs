//! Generated, gate, and macro-implementation lock records.

use serde::{Deserialize, Serialize};

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

/// A macro-providing package bound to its reviewed implementation input set.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LockedMacroImplementation {
    /// Cargo package name that provides the macro implementation.
    pub package: String,
    /// Normalized repository-relative package directory.
    pub directory: String,
    /// Lowercase SHA-256 digest of framed input paths and bytes, including helper packages.
    #[serde(alias = "manifest_sha256")]
    pub inputs_sha256: String,
}
