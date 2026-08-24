//! Invariant evidence and canonical qualification-gate declarations.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
/// Execution context in which a qualification gate is authoritative.
pub enum GateKind {
    /// A deterministic command intended for a developer workstation.
    Local,
    /// A command required in continuous integration.
    Ci,
    /// A command reserved for release qualification.
    Release,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// A named executable check that can serve as invariant evidence.
pub struct GateContract {
    /// Stable gate identity referenced as `gate:<name>`.
    pub name: String,
    /// Execution context in which this gate is authoritative.
    pub kind: GateKind,
    /// Repository-relative regular-file path to the gate executable.
    pub path: String,
    #[serde(default)]
    /// Repository-relative files whose bytes qualify this gate's authority.
    pub inputs: Vec<String>,
    #[serde(default)]
    /// Names of gates that must run before this gate.
    pub requires: Vec<String>,
    /// Human explanation of the architecture property the gate establishes.
    pub reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// An exact production source to named Cargo-test mirror backed by an execution receipt.
pub struct TestMirrorContract {
    /// Repository-relative Rust source that must be production-reachable.
    pub production: String,
    /// Repository-relative Rust test source that must be Cargo-test-reachable.
    pub test: String,
    /// Exact Rust test-function identifier declared in `test`.
    pub name: String,
    /// Repository-relative schema-1 JSON execution receipt.
    pub receipt: String,
    /// Human explanation of why this test mirrors the production source.
    pub reason: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
/// Lifecycle status for a declared architecture invariant.
pub enum InvariantStatus {
    /// The invariant has validated, repository-local evidence.
    Enforced,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// A documented architecture promise backed by tests or gates.
pub struct InvariantContract {
    /// Stable machine-oriented invariant identifier.
    pub id: String,
    /// Short human-readable invariant name.
    pub title: String,
    /// Current enforcement lifecycle state.
    pub status: InvariantStatus,
    /// Repository-relative document containing the invariant definition.
    pub document: String,
    /// Qualified `rust-test:` or `gate:` references proving enforcement.
    pub evidence: Vec<String>,
}
