//! Exact item-macro namespace manifests retained as lock authority.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// One item-macro invocation bound to its reviewed exact namespace manifest.
pub struct LockedItemMacroManifest {
    /// Exact macro policy identity.
    pub name: String,
    /// Exact Rust source file containing the invocation.
    pub invocation_path: String,
    /// Exact checked-in expansion manifest path.
    pub manifest_path: String,
    /// SHA-256 of the exact manifest bytes.
    pub manifest_sha256: String,
    /// SHA-256 of the canonical invocation token stream.
    pub invocation_sha256: String,
    /// Number of exact declarations in the manifest.
    pub bindings: usize,
}
