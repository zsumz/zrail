//! Cargo package directories become conservative initial Rust source boundaries.

mod baseline;

use std::path::Path;

use crate::{
    cargo::load_cargo_workspace, engine::CheckError, inventory::inventory_cargo_repository,
};

pub use baseline::{
    BaselinePlan, BaselineRatchet, BaselineRule, BaselineSize, discover_baseline,
    discover_baseline_rules,
};

/// Discovers conservative Rust source roots from a Cargo repository.
///
/// Package directories are normalized, sorted, and deduplicated. A root package
/// collapses the result to `.` because it already bounds the complete repository.
/// The operation reads manifests but does not invoke Cargo or modify files.
pub fn discover_source_roots(root: &Path) -> Result<Vec<String>, CheckError> {
    let inventory = inventory_cargo_repository(root)
        .map_err(|error| CheckError::from_message(error.to_string()))?;
    let cargo = load_cargo_workspace(&inventory)
        .map_err(|error| CheckError::from_message(error.to_string()))?;
    let mut roots = cargo
        .packages
        .iter()
        .map(|package| package.directory.clone())
        .collect::<Vec<_>>();
    roots.sort();
    roots.dedup();
    if roots.is_empty() {
        return Err(CheckError::from_message(
            "Cargo repository contains no packages",
        ));
    }
    if roots.iter().any(|root| root == ".") {
        Ok(vec![".".into()])
    } else {
        Ok(roots)
    }
}
