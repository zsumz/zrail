//! Namespace-preservation authority binds every exact candidate at one occurrence.

use zrail_core::{AnalysisQuality, Contract, MacroExpansionBindings, MacroExpansionMode};

use crate::{
    cargo::ResolvedCargoGraph,
    source::{BindingMacroPolicy, SourceIndex},
};

use super::{
    allowances::AllowanceIndex,
    review::{MacroBindingResult, review},
};

pub(crate) fn build(
    contract: &Contract,
    source: &SourceIndex,
    resolved_cargo: Option<&ResolvedCargoGraph>,
) -> BindingMacroPolicy {
    let allowed = if contract.source.rust.macros.mode == MacroExpansionMode::Allow {
        AllowanceIndex::new(std::iter::empty())
    } else {
        AllowanceIndex::new(&contract.source.rust.macros.allow)
    };
    let mut policy = BindingMacroPolicy::default();
    for file in source
        .files
        .iter()
        .filter(|file| !file.reachability.is_unreachable())
    {
        for expansion in &file.macro_expansions {
            if expansion.is_compiler_builtin() {
                policy.trust(&file.relative, expansion);
                continue;
            }
            let item_invocation = file
                .item_macros
                .iter()
                .find(|invocation| invocation.span == expansion.span);
            if contract.source.rust.macros.mode == MacroExpansionMode::Allow
                && item_invocation.is_none_or(|invocation| {
                    crate::rules::source_graph::item_macro_is_authorized(
                        contract,
                        file,
                        invocation,
                        resolved_cargo,
                    )
                })
            {
                policy.accept_opaque(&file.relative, expansion);
            }
            let MacroBindingResult::Bound {
                allowances,
                confidence: AnalysisQuality::Exact,
            } = review(source, resolved_cargo, expansion, &allowed)
            else {
                continue;
            };
            if complete_namespace_authority(expansion, &allowances, &allowed) {
                policy.trust(&file.relative, expansion);
            } else {
                policy.accept_opaque(&file.relative, expansion);
            }
        }
        for invocation in &file.item_macros {
            let Some(expansion) = file
                .macro_expansions
                .iter()
                .find(|expansion| expansion.span == invocation.span)
            else {
                continue;
            };
            if crate::rules::source_graph::item_macro_is_authorized(
                contract,
                file,
                invocation,
                resolved_cargo,
            ) {
                policy.accept_opaque(&file.relative, expansion);
                let has_manifest = contract.source.rust.item_macros.iter().any(|allowance| {
                    allowance.manifest.is_some()
                        && allowance.path.as_deref() == Some(file.relative.as_str())
                        && invocation.policy_names().any(|name| name == allowance.name)
                });
                if has_manifest {
                    policy.trust(&file.relative, expansion);
                }
            }
        }
    }
    policy
}

fn complete_namespace_authority(
    expansion: &crate::source::MacroExpansionFact,
    allowances: &[&zrail_core::MacroExpansionAllow],
    allowed: &AllowanceIndex<'_>,
) -> bool {
    !allowances.is_empty()
        && allowances.iter().all(|allowance| clean(allowance))
        && expansion.candidates.iter().all(|candidate| {
            candidate.policy_names().all(|name| {
                allowed
                    .get(name)
                    .is_some_and(|entries| entries.iter().any(|allowance| clean(allowance)))
            })
        })
}

fn clean(allowance: &zrail_core::MacroExpansionAllow) -> bool {
    allowance.bindings == MacroExpansionBindings::None
}

#[cfg(test)]
#[path = "binding_policy_test.rs"]
mod binding_policy_test;

#[cfg(test)]
pub(super) use binding_policy_test::{clean_allowance, contract, expansion, source};
