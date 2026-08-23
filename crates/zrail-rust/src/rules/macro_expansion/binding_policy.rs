//! Namespace-preservation authority binds every exact candidate at one occurrence.

use std::collections::BTreeMap;

use zrail_core::{AnalysisQuality, Contract, MacroExpansionBindings, MacroExpansionMode};

use crate::source::{BindingMacroPolicy, SourceIndex};

use super::review::{MacroBindingResult, review};

pub(crate) fn build(contract: &Contract, source: &SourceIndex) -> BindingMacroPolicy {
    if contract.source.rust.macros.mode == MacroExpansionMode::Allow {
        return BindingMacroPolicy::default();
    }
    let allowed = contract
        .source
        .rust
        .macros
        .allow
        .iter()
        .map(|allowance| (allowance.name.as_str(), allowance))
        .collect::<BTreeMap<_, _>>();
    let mut policy = BindingMacroPolicy::default();
    for file in source
        .files
        .iter()
        .filter(|file| !file.reachability.is_unreachable())
    {
        for expansion in &file.macro_expansions {
            let MacroBindingResult::Bound {
                allowances,
                confidence: AnalysisQuality::Exact,
            } = review(source, expansion, &allowed)
            else {
                continue;
            };
            if complete_namespace_authority(expansion, &allowances, &allowed) {
                policy.trust(&file.relative, expansion);
            }
        }
    }
    policy
}

fn complete_namespace_authority(
    expansion: &crate::source::MacroExpansionFact,
    allowances: &[&str],
    allowed: &BTreeMap<&str, &zrail_core::MacroExpansionAllow>,
) -> bool {
    !allowances.is_empty()
        && allowances.iter().all(|name| clean(allowed[*name]))
        && expansion.candidates.iter().all(|candidate| {
            candidate
                .policy_names()
                .all(|name| allowed.get(name).is_some_and(|allowance| clean(allowance)))
        })
}

fn clean(allowance: &zrail_core::MacroExpansionAllow) -> bool {
    allowance.bindings == MacroExpansionBindings::None
}

#[cfg(test)]
#[path = "binding_policy_test.rs"]
mod binding_policy_test;
