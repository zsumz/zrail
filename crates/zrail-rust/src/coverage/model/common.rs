//! Exact Cargo-domain and feature-world coverage records.

use serde::Serialize;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
/// One exact Cargo target compilation domain.
pub struct GovernedCompilationDomain {
    /// Cargo package owning the target.
    pub package: String,
    /// Rust edition used by the target.
    pub edition: String,
    /// Cargo target name.
    pub target: String,
    /// Compilation mode in kebab-case.
    pub mode: String,
    /// Configured exact feature world, or `None` for legacy conditional analysis.
    pub feature_world: Option<String>,
    /// Exact active package features in this compilation domain.
    pub features: Vec<String>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
/// One configured workspace-wide exact Cargo feature world.
pub struct GovernedFeatureWorld {
    /// Stable contract-authored world name.
    pub name: String,
    /// Complete workspace package feature selections and resolved closures.
    pub packages: Vec<GovernedFeaturePackage>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
/// One package's authored and resolved state inside an exact feature world.
pub struct GovernedFeaturePackage {
    /// Workspace package name.
    pub package: String,
    /// Whether the package's default feature was selected.
    pub default_features: bool,
    /// Exact directly selected feature set.
    pub selected: Vec<String>,
    /// Exact fixed-point active feature closure.
    pub active: Vec<String>,
}
