//! Item-producing macro authority follows path scope and optional origin binding.

mod binding;

use std::collections::BTreeSet;

use zrail_core::{Contract, ItemMacroContract, glob_matches};

use crate::{
    cargo::ResolvedCargoGraph,
    source::{MacroExpansionFact, ObservedFact, RustFileFacts, SourceIndex},
};

pub(super) fn review(
    contract: &Contract,
    source: &SourceIndex,
    resolved_cargo: Option<&ResolvedCargoGraph>,
) -> Vec<zrail_core::Finding> {
    let mut findings = Vec::new();
    let mut used = BTreeSet::new();
    for file in source
        .files
        .iter()
        .filter(|file| !file.reachability.is_unreachable())
    {
        for invocation in &file.item_macros {
            let authorities =
                matching_authorities(contract, &file.relative, file, invocation, resolved_cargo);
            if authorities.is_empty() && !directly_inspected(file, invocation) {
                findings.push(unresolved(file, invocation));
            } else {
                used.extend(authorities);
            }
        }
    }
    for (index, allowance) in contract.source.rust.item_macros.iter().enumerate() {
        if !used.contains(&index) {
            findings.push(stale(allowance));
        }
    }
    findings
}

pub(super) fn matching_authorities(
    contract: &Contract,
    path: &str,
    file: &RustFileFacts,
    invocation: &ObservedFact,
    resolved_cargo: Option<&ResolvedCargoGraph>,
) -> Vec<usize> {
    let expansion = expansion_for(file, invocation);
    contract
        .source
        .rust
        .item_macros
        .iter()
        .enumerate()
        .filter(|(_, allowance)| selects(allowance, path))
        .filter(|(_, allowance)| name_matches(allowance, invocation, expansion))
        .filter(|(_, allowance)| binding::matches(allowance, expansion, resolved_cargo))
        .map(|(index, _)| index)
        .collect()
}

pub(crate) fn authorities_for_file(
    contract: &Contract,
    file: &RustFileFacts,
    resolved_cargo: Option<&ResolvedCargoGraph>,
) -> Vec<usize> {
    file.item_macros
        .iter()
        .flat_map(|invocation| {
            matching_authorities(contract, &file.relative, file, invocation, resolved_cargo)
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub(crate) fn selector_name(allowance: &ItemMacroContract) -> String {
    match &allowance.path {
        Some(path) => format!("at {path}"),
        None if allowance.within.is_empty() => "repository-wide".into(),
        None => format!("within [{}]", allowance.within.join(", ")),
    }
}

fn selects(allowance: &ItemMacroContract, path: &str) -> bool {
    allowance.path.as_ref().map_or_else(
        || {
            allowance.within.is_empty()
                || allowance
                    .within
                    .iter()
                    .any(|pattern| glob_matches(pattern, path))
        },
        |exact| exact == path,
    )
}

fn name_matches(
    allowance: &ItemMacroContract,
    invocation: &ObservedFact,
    expansion: Option<&MacroExpansionFact>,
) -> bool {
    invocation.policy_names().any(|name| name == allowance.name)
        || expansion.is_some_and(|expansion| {
            expansion.candidates.iter().any(|candidate| {
                candidate
                    .allowance_names(&expansion.name)
                    .contains(&allowance.name.as_str())
            })
        })
}

fn expansion_for<'a>(
    file: &'a RustFileFacts,
    invocation: &ObservedFact,
) -> Option<&'a MacroExpansionFact> {
    file.macro_expansions
        .iter()
        .find(|expansion| expansion.span == invocation.span)
}

fn directly_inspected(file: &RustFileFacts, invocation: &ObservedFact) -> bool {
    expansion_for(file, invocation).is_some_and(|expansion| {
        expansion.quality == zrail_core::AnalysisQuality::Exact
            && expansion.is_compiler_builtin()
            && expansion
                .candidates
                .iter()
                .flat_map(crate::source::MacroCandidate::policy_names)
                .all(|name| name.rsplit("::").next() == Some("thread_local"))
    })
}

fn unresolved(file: &RustFileFacts, invocation: &ObservedFact) -> zrail_core::Finding {
    zrail_core::Finding::error(
        "RUST-GRAPH-003",
        "rust.source-graph.analysis",
        "source-graph",
        format!(
            "item-position macro {}! may create source edges that static analysis cannot resolve",
            invocation.name
        ),
    )
    .at(&file.relative, invocation.span)
    .with_analysis(zrail_core::AnalysisQuality::Unresolved)
    .with_help("replace the boundary with a literal repository-local .rs source path")
}

fn stale(allowance: &ItemMacroContract) -> zrail_core::Finding {
    let finding = zrail_core::Finding::error(
        "RUST-GRAPH-005",
        "rust.source-graph.item-macro",
        "source-graph",
        format!(
            "item macro authority {}! {} matches no reachable invocation",
            allowance.name,
            selector_name(allowance)
        ),
    )
    .because(&allowance.reason)
    .with_help("remove stale item-macro authority or restore an in-scope invocation");
    if let Some(path) = &allowance.path {
        finding.at(path, None)
    } else {
        finding
    }
}
