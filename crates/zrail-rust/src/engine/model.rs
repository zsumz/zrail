//! Loading the shared repository fact model once per command.

use std::path::{Path, PathBuf};

use zrail_core::{ContractBundle, load_contract, repository_file};

use crate::{
    cargo::{CargoWorkspace, apply_attestations, load_cargo_workspace},
    inventory::{RepositoryInventory, inventory_repository},
    rules::source_graph,
    source::{ResolvedModuleEdge, SourceIndex, canonicalize_dependency_roots, index_rust_source},
};

use super::CheckError;

#[derive(Debug)]
pub(crate) struct RepositoryModel {
    pub(crate) bundle: ContractBundle,
    pub(crate) inventory: RepositoryInventory,
    pub(crate) cargo: CargoWorkspace,
    pub(crate) source: SourceIndex,
    pub(crate) module_edges: Vec<ResolvedModuleEdge>,
}

pub(crate) fn load_model(root: &Path, config: &Path) -> Result<RepositoryModel, CheckError> {
    let bundle =
        load_contract(root, config).map_err(|error| CheckError::from_message(error.to_string()))?;
    load_model_with_bundle(root, bundle)
}

pub(crate) fn load_model_with_bundle(
    root: &Path,
    bundle: ContractBundle,
) -> Result<RepositoryModel, CheckError> {
    let mut inventory = inventory_repository(root, &bundle.contract)
        .map_err(|error| CheckError::from_message(error.to_string()))?;
    let mut cargo = load_cargo_workspace(&inventory)
        .map_err(|error| CheckError::from_message(error.to_string()))?;
    apply_attestations(&mut cargo, &bundle.contract.dependencies.crate_roots);
    inventory
        .rust_files
        .retain(|file| cargo.source_is_active(&file.relative));
    let mut source = index_rust_source(&inventory, &bundle.contract.source.rust);
    let graph = source_graph::analyze(&bundle.contract, &inventory, &cargo, &source);
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
    canonicalize_dependency_roots(
        &mut source,
        &cargo,
        &graph.packages,
        &graph.module_edges,
        &graph.compilation_domains,
        &graph.compilation_edges,
    );
    source.findings.extend(graph.findings);
    let item_macro_findings = source_graph::review_item_macros(&bundle.contract, &source);
    source.findings.extend(item_macro_findings);
    Ok(RepositoryModel {
        bundle,
        inventory,
        cargo,
        source,
        module_edges: graph.module_edges,
    })
}

pub(crate) fn resolve(root: &Path, path: &Path) -> Result<PathBuf, CheckError> {
    repository_file(root, path).map_err(CheckError::from_message)
}
