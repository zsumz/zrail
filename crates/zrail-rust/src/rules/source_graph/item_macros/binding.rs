//! Exact origin matching for item-macro authority.

use zrail_core::{
    CrateRootSource, ItemMacroContract, MacroExpansionAllow, MacroExpansionBindings, MacroInputMode,
};

use crate::{
    cargo::{ResolvedCargoGraph, source_matches},
    rules::macro_expansion,
    source::{MacroExpansionFact, MacroOrigin},
};

pub(super) fn matches(
    allowance: &ItemMacroContract,
    expansion: Option<&MacroExpansionFact>,
    resolved_cargo: Option<&ResolvedCargoGraph>,
) -> bool {
    let Some(binding) = allowance.binding else {
        return true;
    };
    let Some(expansion) = expansion else {
        return false;
    };
    let macro_allowance = MacroExpansionAllow {
        name: allowance.name.clone(),
        inputs: MacroInputMode::Inspect,
        binding,
        bindings: MacroExpansionBindings::Opaque,
        async_syntax: zrail_core::MacroAsyncSyntax::Opaque,
        duplication_effect: zrail_core::MacroDuplicationEffect::Opaque,
        source_operations: zrail_core::MacroSourceOperations::Opaque,
        field_mutation: zrail_core::MacroFieldMutation::Opaque,
        definition: None,
        source: allowance.source.clone(),
        reason: allowance.reason.clone(),
    };
    macro_expansion::binds_allowance(expansion, &macro_allowance)
        && expansion.candidates.iter().all(|candidate| {
            candidate.origins.iter().all(|origin| match origin {
                MacroOrigin::CompilerBuiltin => allowance.source.is_none(),
                MacroOrigin::Repository { package, directory } => {
                    repository_definition_matches(allowance, candidate, package, directory)
                }
                MacroOrigin::External { package, source } => {
                    allowance.source.as_ref().is_some_and(|allowed| {
                        external_source_matches(allowed, package, source, resolved_cargo)
                    })
                }
                MacroOrigin::Pending { .. } | MacroOrigin::Unresolved => false,
            })
        })
}

fn repository_definition_matches(
    allowance: &ItemMacroContract,
    candidate: &crate::source::MacroCandidate,
    package: &str,
    directory: &str,
) -> bool {
    match allowance.source.as_ref() {
        None => candidate.definition.is_some() && candidate.definition_sha256.is_some(),
        Some(CrateRootSource::Repository {
            package: allowed_package,
            directory: allowed_directory,
            ..
        }) => allowed_package == package && allowed_directory == directory,
        Some(_) => false,
    }
}

fn external_source_matches(
    allowed: &CrateRootSource,
    package: &str,
    observed: &crate::cargo::DependencySource,
    resolved_cargo: Option<&ResolvedCargoGraph>,
) -> bool {
    let CrateRootSource::CargoLock {
        package: selected,
        version,
        source,
    } = allowed
    else {
        return source_matches(allowed, observed);
    };
    let Some(graph) = resolved_cargo else {
        return false;
    };
    let Ok(selected) = graph.lookup(selected, version.as_deref(), source.as_deref()) else {
        return false;
    };
    graph
        .package_for_source(package, observed)
        .is_ok_and(|observed| observed == selected)
}
