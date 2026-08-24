//! Loading the shared repository fact model once per command.

use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

use zrail_core::{ContractBundle, load_contract, repository_file};

use crate::{
    cargo::{
        CargoWorkspace, ResolvedCargoGraph, apply_attestations, load_cargo_workspace,
        validate_resolved_sources,
    },
    inventory::{RepositoryInventory, inventory_repository},
    rules::source_graph,
    source::{
        CanonicalizationContext, CompilationDomain, ResolvedModuleEdge, SourceIndex,
        canonicalize_dependency_roots, index_rust_source,
    },
};

use super::CheckError;

#[derive(Debug)]
pub(crate) struct RepositoryModel {
    pub(crate) bundle: ContractBundle,
    pub(crate) inventory: RepositoryInventory,
    pub(crate) cargo: CargoWorkspace,
    pub(crate) resolved_cargo: Option<ResolvedCargoGraph>,
    pub(crate) source: SourceIndex,
    pub(crate) item_macro_manifests: Vec<zrail_core::LockedItemMacroManifest>,
    pub(crate) compilation_domains: BTreeMap<String, BTreeSet<CompilationDomain>>,
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
    let resolved_cargo = ResolvedCargoGraph::load(&inventory.root, &cargo.packages)
        .map_err(|error| CheckError::from_message(error.to_string()))?;
    validate_resolved_sources(resolved_cargo.as_ref(), &bundle.contract)
        .map_err(|error| CheckError::from_message(error.to_string()))?;
    apply_attestations(&mut cargo, &bundle.contract.dependencies.crate_roots);
    inventory
        .rust_files
        .retain(|file| cargo.source_is_active(&file.relative));
    let mut source = index_rust_source(&inventory, &bundle.contract.source.rust);
    let applied_item_macro_manifests =
        super::item_macro_manifests::apply(&inventory, &bundle.contract, &mut source)?;
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
        CanonicalizationContext {
            cargo: &cargo,
            packages: &graph.packages,
            module_edges: &graph.module_edges,
            compilation_domains: &graph.compilation_domains,
            compilation_roots: &graph.compilation_roots,
            compilation_edges: &graph.compilation_edges,
            compilation_includes: &graph.compilation_includes,
            analysis_limits: &bundle.contract.analysis.limits,
        },
        |source| {
            crate::rules::binding_macro_policy(&bundle.contract, source, resolved_cargo.as_ref())
        },
    );
    source.findings.extend(graph.findings);
    let item_macro_findings =
        source_graph::review_item_macros(&bundle.contract, &source, resolved_cargo.as_ref());
    source.findings.extend(item_macro_findings);
    let item_macro_manifests = super::item_macro_manifests::locked(
        applied_item_macro_manifests,
        &bundle.contract,
        &source,
        resolved_cargo.as_ref(),
        &graph.compilation_domains,
    )?;
    Ok(RepositoryModel {
        bundle,
        inventory,
        cargo,
        resolved_cargo,
        source,
        item_macro_manifests,
        compilation_domains: graph.compilation_domains,
        module_edges: graph.module_edges,
    })
}

pub(crate) fn resolve(root: &Path, path: &Path) -> Result<PathBuf, CheckError> {
    repository_file(root, path).map_err(CheckError::from_message)
}
