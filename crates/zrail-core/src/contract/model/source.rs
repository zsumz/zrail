//! Typed Rust source-role override policy.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
/// Effective source role selected for one exact path.
pub enum FileRole {
    /// Enforce declarative facade shape and facade size budgets.
    Facade,
    /// Enforce implementation size budgets without facade shape restrictions.
    Implementation,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// One reasoned exact override of an inferred Rust source role.
pub struct FileRoleContract {
    /// Exact normalized repository-relative Rust source path.
    pub path: String,
    /// Effective source role selected for the path.
    pub role: FileRole,
    /// Human justification for overriding the inferred source role.
    pub reason: String,
}
