//! Cargo.lock-resolved authority for an allowed macro expansion.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// One macro allowance bound to an immutable Cargo.lock package node.
pub struct LockedMacroSource {
    /// Macro allowance identity from the governing contract.
    pub allowance: String,
    /// Exact Cargo package name.
    pub package: String,
    /// Exact resolved semantic version.
    pub version: String,
    /// Exact Cargo.lock source identity.
    pub source: String,
    /// Registry checksum when Cargo.lock supplies one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,
}

#[cfg(test)]
#[path = "macro_source_test.rs"]
mod macro_source_test;
