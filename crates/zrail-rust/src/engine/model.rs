//! Loading the shared repository fact model once per command.

use std::path::{Path, PathBuf};

use zrail_core::{ContractBundle, load_contract, path::repository_file};

use crate::{
    cargo::{CargoWorkspace, load_cargo_workspace},
    inventory::{RepositoryInventory, inventory_repository},
    rules::source_graph,
    source::{SourceIndex, canonicalize_dependency_roots, index_rust_source},
};

use super::CheckError;

#[derive(Debug)]
pub(crate) struct RepositoryModel {
    pub(crate) bundle: ContractBundle,
    pub(crate) inventory: RepositoryInventory,
    pub(crate) cargo: CargoWorkspace,
    pub(crate) source: SourceIndex,
}

pub(crate) fn load_model(root: &Path, config: &Path) -> Result<RepositoryModel, CheckError> {
    let bundle =
        load_contract(root, config).map_err(|error| CheckError::from_message(error.to_string()))?;
    let inventory = inventory_repository(root, &bundle.contract)
        .map_err(|error| CheckError::from_message(error.to_string()))?;
    let cargo = load_cargo_workspace(&inventory)
        .map_err(|error| CheckError::from_message(error.to_string()))?;
    let mut source = index_rust_source(&inventory);
    let graph = source_graph::analyze(&bundle.contract, &inventory, &cargo, &source);
    canonicalize_dependency_roots(&mut source, &cargo, &graph.packages);
    for file in &mut source.files {
        file.packages = graph
            .packages
            .get(&file.relative)
            .map(|packages| packages.iter().cloned().collect())
            .unwrap_or_default();
        file.reachability = graph
            .reachability
            .get(&file.relative)
            .copied()
            .unwrap_or_default();
    }
    source.findings.extend(graph.findings);
    Ok(RepositoryModel {
        bundle,
        inventory,
        cargo,
        source,
    })
}

pub(crate) fn resolve(root: &Path, path: &Path) -> Result<PathBuf, CheckError> {
    repository_file(root, path).map_err(CheckError::from_message)
}
