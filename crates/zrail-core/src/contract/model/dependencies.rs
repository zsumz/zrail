//! Dependency-policy schema, including reviewed external crate-root attestations.

use serde::{Deserialize, Serialize};

use crate::contract::{CycleMode, DependencyMode, PolicyMode};

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DependenciesContract {
    pub mode: DependencyMode,
    pub unassigned_packages: PolicyMode,
    pub cycles: CycleMode,
    #[serde(default, rename = "crate_root")]
    pub crate_roots: Vec<CrateRootContract>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CrateRootContract {
    pub package: String,
    pub root: String,
    pub reason: String,
}
