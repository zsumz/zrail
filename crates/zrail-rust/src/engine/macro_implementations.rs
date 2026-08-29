//! Repository macro trust binds to a bounded deterministic package input manifest.

mod manifest;

use std::collections::BTreeSet;

use zrail_core::LockedMacroImplementation;

use crate::{
    cargo::CargoWorkspace,
    inventory::RepositoryInventory,
    source::{MacroOrigin, SourceIndex},
};

use super::{CheckError, model::RepositoryModel};
use manifest::repository_manifest;

#[cfg(test)]
use manifest::{MAX_IMPLEMENTATION_INPUTS, digest_inputs};

pub(super) fn locked(
    model: &RepositoryModel,
) -> Result<Vec<LockedMacroImplementation>, CheckError> {
    locked_for_sources(
        &model.bundle.contract,
        &model.inventory,
        &model.cargo,
        &model.source,
    )
}

pub(super) fn locked_for_sources(
    contract: &zrail_core::Contract,
    inventory: &RepositoryInventory,
    cargo: &CargoWorkspace,
    source: &SourceIndex,
) -> Result<Vec<LockedMacroImplementation>, CheckError> {
    let packages = trusted_packages(contract, source);
    packages
        .into_iter()
        .map(|(package, directory)| {
            repository_manifest(inventory, cargo, source, &package, &directory)
        })
        .collect()
}

fn trusted_packages(
    contract: &zrail_core::Contract,
    source: &SourceIndex,
) -> BTreeSet<(String, String)> {
    let allowed = contract
        .source
        .rust
        .macros
        .allow
        .iter()
        .map(|allowance| allowance.name.as_str())
        .chain(
            contract
                .source
                .rust
                .item_macros
                .iter()
                .filter(|allowance| {
                    matches!(
                        allowance.source.as_ref(),
                        Some(zrail_core::CrateRootSource::Repository { .. })
                    )
                })
                .map(|allowance| allowance.name.as_str()),
        )
        .collect::<BTreeSet<_>>();
    source
        .files
        .iter()
        .filter(|file| !file.reachability.is_unreachable())
        .flat_map(|file| &file.macro_expansions)
        .filter(|expansion| expansion.names_covered_by(&allowed))
        .flat_map(crate::source::MacroExpansionFact::origins)
        .filter_map(|origin| match origin {
            MacroOrigin::Repository { package, directory } => {
                Some((package.clone(), directory.clone()))
            }
            _ => None,
        })
        .collect()
}

#[cfg(test)]
#[path = "macro_implementations_test.rs"]
mod macro_implementations_test;
