//! Test-mirror coverage records.

use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
/// One exact production-to-test mirror declaration.
pub struct GovernedTestMirror {
    /// Canonical report identity for this mirror.
    pub policy_id: String,
    /// Production-reachable Rust source path.
    pub production: String,
    /// Cargo-test-reachable Rust source path.
    pub test: String,
    /// Exact test function identifier.
    pub test_name: String,
    /// Repository-relative execution receipt path.
    pub receipt: String,
    /// Additional exact files bound into the execution receipt.
    pub inputs: Vec<String>,
    /// Exact producer-asserted execution command.
    pub command: String,
    /// Cargo package selected by the execution command.
    pub package: String,
    /// Whether Cargo default features were enabled.
    pub default_features: bool,
    /// Exact enabled Cargo feature set.
    pub features: Vec<String>,
    /// Exact compilation target triple.
    pub target: String,
    /// Normalized Rust toolchain identity.
    pub toolchain: String,
    /// Contract-authored justification.
    pub reason: String,
}
