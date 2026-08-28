//! Literal include targets are indexed under their exact syntactic position.

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::Contract;

use crate::{
    inventory::{RepositoryInventory, load_referenced_source},
    source::{
        IncludeContext, SourceIndex, SourceSyntax, index_rust_source_with_hints, join_relative,
        parent,
    },
};

use super::CheckError;

pub(super) fn index(
    inventory: &mut RepositoryInventory,
    contract: &Contract,
) -> Result<SourceIndex, CheckError> {
    let mut hints = BTreeMap::<String, BTreeSet<SourceSyntax>>::new();
    let mut source_bytes = inventory
        .rust_files
        .iter()
        .map(|file| file.source.len())
        .sum();
    let mut loaded = inventory
        .rust_files
        .iter()
        .map(|file| file.relative.clone())
        .collect::<BTreeSet<_>>();
    loop {
        let source = index_rust_source_with_hints(inventory, &contract.source.rust, &hints);
        let mut changed = false;
        for file in &source.files {
            if hints
                .entry(file.relative.clone())
                .or_default()
                .insert(file.syntax)
            {
                changed = true;
            }
            for include in &file.includes {
                let Some(relative) = &include.path else {
                    continue;
                };
                let Ok(target) = join_relative(&parent(&file.relative), relative) else {
                    continue;
                };
                let syntax = syntax(include.context);
                if hints.entry(target.clone()).or_default().insert(syntax) {
                    changed = true;
                }
                if loaded.insert(target.clone()) {
                    changed |=
                        load_referenced_source(inventory, contract, &target, &mut source_bytes)
                            .map_err(|error| CheckError::from_message(error.to_string()))?;
                }
            }
        }
        if !changed {
            return Ok(source);
        }
    }
}

const fn syntax(context: IncludeContext) -> SourceSyntax {
    match context {
        IncludeContext::Items => SourceSyntax::Items,
        IncludeContext::Expression => SourceSyntax::Expression,
        IncludeContext::ImplItems => SourceSyntax::ImplItems,
        IncludeContext::TraitItems => SourceSyntax::TraitItems,
    }
}
