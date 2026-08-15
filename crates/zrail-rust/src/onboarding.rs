//! Cargo package directories become conservative initial Rust source boundaries.

mod baseline;

use std::path::Path;

use crate::{
    cargo::load_cargo_workspace, engine::CheckError, inventory::inventory_cargo_repository,
};

pub use baseline::{BaselinePlan, BaselineRatchet, discover_baseline};

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
