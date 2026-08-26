//! Entry points that expose source-graph traversal and macro authority.

use zrail_core::{Contract, Finding, ItemMacroContract};

use crate::{
    cargo::{CargoWorkspace, ResolvedCargoGraph, ResolvedFeatureWorld},
    inventory::RepositoryInventory,
    source::{ObservedFact, RustFileFacts, SourceIndex},
};

use super::{SourceGraphAnalysis, item_macros, walker::Walker};

pub(crate) fn analyze(
    contract: &Contract,
    inventory: &RepositoryInventory,
    cargo: &CargoWorkspace,
    feature_worlds: &[ResolvedFeatureWorld],
    source: &SourceIndex,
) -> SourceGraphAnalysis {
    Walker::new(contract, inventory, cargo, feature_worlds, source).run()
}

pub(crate) fn item_macro_authorities(
    contract: &Contract,
    file: &RustFileFacts,
    resolved_cargo: Option<&ResolvedCargoGraph>,
) -> Vec<usize> {
    item_macros::authorities_for_file(contract, file, resolved_cargo)
}

pub(crate) fn item_macro_is_authorized(
    contract: &Contract,
    file: &RustFileFacts,
    invocation: &ObservedFact,
    resolved_cargo: Option<&ResolvedCargoGraph>,
) -> bool {
    !item_macros::matching_authorities(contract, &file.relative, file, invocation, resolved_cargo)
        .is_empty()
}

pub(crate) fn item_macro_selector(allowance: &ItemMacroContract) -> String {
    item_macros::selector_name(allowance)
}

pub(crate) fn review_item_macros(
    contract: &Contract,
    source: &SourceIndex,
    resolved_cargo: Option<&ResolvedCargoGraph>,
) -> Vec<Finding> {
    item_macros::review(contract, source, resolved_cargo)
}
