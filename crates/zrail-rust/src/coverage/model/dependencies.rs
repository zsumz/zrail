//! Dependency-policy coverage records.

use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
/// One configured package dependency prohibition.
pub struct GovernedDependencyRule {
    /// Canonical report identity for this policy.
    pub policy_id: String,
    /// Contract-authored rule name.
    pub name: String,
    /// Exact workspace package selected as the path origin.
    pub from: String,
    /// Denied resolved package names in canonical order.
    pub deny: Vec<String>,
    /// Direct or transitive graph reachability.
    pub reachability: String,
    /// Effective first-edge dependency kinds in canonical order.
    pub kinds: Vec<String>,
    /// Contract-authored justification.
    pub reason: String,
    /// Shortest exact resolved paths reaching denied packages.
    pub paths: Vec<GovernedDependencyPath>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
/// One shortest prohibited path through exact Cargo.lock nodes.
pub struct GovernedDependencyPath {
    /// Kind of the first manifest edge entering the resolved path.
    pub kind: String,
    /// Denied package name reached by this path.
    pub denied: String,
    /// Ordered exact package identities from workspace root to prohibition.
    pub nodes: Vec<GovernedPackageIdentity>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
/// One immutable package identity parsed from Cargo.lock.
pub struct GovernedPackageIdentity {
    /// Cargo package name.
    pub name: String,
    /// Exact locked package version.
    pub version: String,
    /// Exact Cargo.lock source or repository-local path identity.
    pub source: String,
    /// Exact Cargo.lock checksum, when present.
    pub checksum: Option<String>,
}
