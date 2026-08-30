//! Repository macro trust binds to a bounded deterministic package input manifest.

mod manifest;

use std::collections::BTreeSet;

use zrail_core::LockedMacroImplementation;

use crate::{
    cargo::CargoWorkspace,
    inventory::{RepositoryInventory, inventory_selected_cargo_repository},
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
    if packages.is_empty() {
        return Ok(Vec::new());
    }
    // Source exclusions cannot hide compile-effective provider inputs.
    let inputs_inventory = inventory_selected_cargo_repository(&inventory.root, &[])
        .map_err(|error| CheckError::from_message(error.to_string()))?;
    packages
        .into_iter()
        .map(|(package, directory)| {
            let inputs = contract
                .source
                .rust
                .macros
                .allow
                .iter()
                .filter_map(|allowance| allowance.source.as_ref())
                .chain(
                    contract
                        .source
                        .rust
                        .item_macros
                        .iter()
                        .filter_map(|allowance| allowance.source.as_ref()),
                )
                .filter_map(|authority| match authority {
                    zrail_core::CrateRootSource::Repository {
                        package: selected,
                        directory: root,
                        inputs,
                        ..
                    } if selected == &package && root == &directory => Some(inputs),
                    _ => None,
                })
                .flatten()
                .cloned()
                .collect::<BTreeSet<_>>();
            repository_manifest(
                &inputs_inventory,
                cargo,
                source,
                &package,
                &directory,
                &inputs,
            )
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
