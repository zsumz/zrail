//! Explicit workspace-wide Cargo feature compilation worlds.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// One named Cargo feature world whose relevant compilation contexts converge.
pub struct CargoFeatureWorldContract {
    /// Stable world name retained in compilation-domain and lock identity.
    pub name: String,
    /// Complete per-package feature selections shared by every relevant context.
    pub packages: Vec<CargoFeaturePackageContract>,
    /// Human explanation of the compilation world represented here.
    pub reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// Exact feature selection for one package inside a named feature world.
pub struct CargoFeaturePackageContract {
    /// Exact Cargo package name.
    pub package: String,
    /// Whether the package's declared default feature is selected.
    #[serde(default = "default_true")]
    pub default_features: bool,
    /// Explicit selected feature names before local and workspace closure.
    #[serde(default)]
    pub features: Vec<String>,
}

const fn default_true() -> bool {
    true
}
