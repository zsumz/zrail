//! Exact whole-Cargo.lock inventories are independent of manifest reachability.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// One reasoned inventory of every Cargo.lock node with an exact package name.
pub struct LockPackageRule {
    /// Unique authored name; the canonical policy ID is `dependency:lock-package:<name>`.
    pub name: String,
    /// Literal Cargo package name, including nodes unreachable from current manifests.
    pub package: String,
    /// Exact quantity or complete immutable identity set required for this name.
    pub assertion: LockPackageAssertion,
    /// Human justification for the inventory.
    pub reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
/// Closed inventory predicates; both modes require a complete parsed Cargo.lock.
pub enum LockPackageAssertion {
    /// Require exactly this many distinct locked nodes; zero is a persistent prohibition.
    Count {
        /// Required number of matching nodes, independent of dependency edge counts.
        count: usize,
    },
    /// Require exactly these version/source/checksum identities, in any order.
    Exact {
        /// Complete expected set; an empty set prohibits every node with the selected name.
        identities: Vec<LockPackageIdentity>,
    },
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// Exact locked identity within the package name selected by a rule.
pub struct LockPackageIdentity {
    /// Exact locked version string, never a version requirement.
    pub version: String,
    /// Exact Cargo source, or `path+<workspace-directory>` for a local workspace node.
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Exact lowercase SHA-256; omission requires that the locked node has no checksum.
    pub checksum: Option<String>,
}
