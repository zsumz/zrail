//! Invariant evidence and canonical qualification-gate declarations.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum GateKind {
    Local,
    Ci,
    Release,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GateContract {
    pub name: String,
    pub kind: GateKind,
    pub path: String,
    #[serde(default)]
    pub inputs: Vec<String>,
    #[serde(default)]
    pub requires: Vec<String>,
    pub reason: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum InvariantStatus {
    Enforced,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InvariantContract {
    pub id: String,
    pub title: String,
    pub status: InvariantStatus,
    pub document: String,
    pub evidence: Vec<String>,
}
