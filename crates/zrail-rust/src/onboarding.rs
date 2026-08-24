//! Cargo package directories become conservative initial Rust source boundaries.

mod baseline;
mod selection;

use std::path::Path;

use crate::{
    cargo::load_cargo_workspace, engine::CheckError, inventory::inventory_selected_cargo_repository,
};

pub use baseline::{
    BaselinePlan, BaselineRatchet, BaselineRule, BaselineSize, discover_baseline,
    discover_baseline_rules,
};
pub use selection::RepositorySelection;

/// Discovers conservative Rust source roots from a Cargo repository.
///
/// Package directories are normalized, sorted, and deduplicated. A root package
/// collapses the result to `.` because it already bounds the complete repository.
/// The operation reads manifests but does not invoke Cargo or modify files.
pub fn discover_source_roots(root: &Path) -> Result<Vec<String>, CheckError> {
    discover_source_roots_with_selection(root, &RepositorySelection::default())
}

/// Discovers conservative Rust roots after applying reviewed repository exclusions.
///
/// Excluded subtrees are not inventoried or parsed. An exclusion may not hide an
/// active Cargo target because that would narrow the initialized authority surface.
pub fn discover_source_roots_with_selection(
    root: &Path,
    selection: &RepositorySelection,
) -> Result<Vec<String>, CheckError> {
    let inventory = inventory_selected_cargo_repository(root, selection.exclusions())
        .map_err(|error| CheckError::from_message(error.to_string()))?;
    let cargo = load_cargo_workspace(&inventory)
        .map_err(|error| CheckError::from_message(error.to_string()))?;
    ensure_targets_selected(selection, &cargo)?;
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

fn ensure_targets_selected(
    selection: &RepositorySelection,
    cargo: &crate::cargo::CargoWorkspace,
) -> Result<(), CheckError> {
    for package in &cargo.packages {
        for target in &package.targets {
            let path = if package.directory == "." {
                target.path.clone()
            } else {
                format!("{}/{path}", package.directory, path = target.path)
            };
            if let Some(pattern) = selection.matching_exclusion(&path) {
                return Err(CheckError::from_message(format!(
                    "repository exclusion {pattern:?} hides authoritative Cargo {:?} target {:?} at {path:?}",
                    target.kind, target.name
                )));
            }
        }
    }
    Ok(())
}
