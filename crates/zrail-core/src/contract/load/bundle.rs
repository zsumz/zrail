//! Validated contract policy paired with its exact source authority.

#[derive(Clone, Debug, Eq, PartialEq)]
/// Exact source bytes contributing to a loaded contract digest.
pub struct ContractSource {
    /// Normalized repository-relative path to the source file.
    pub path: String,
    /// UTF-8 file content exactly as read from disk.
    pub content: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// A validated merged contract together with its source authority.
pub struct ContractBundle {
    /// Typed policy produced by deterministic import merging and validation.
    pub contract: super::super::Contract,
    /// Contributing source files sorted by repository-relative path.
    pub sources: Vec<ContractSource>,
    /// Lowercase SHA-256 digest of every source path and exact source byte string.
    pub sha256: String,
}
