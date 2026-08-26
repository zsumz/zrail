//! Closed macro-policy predicates shared by enforcement and coverage.

use std::collections::BTreeMap;

use zrail_core::{AnalysisQuality, MacroAsyncSyntax, MacroDuplicationEffect, MacroExpansionAllow};

use crate::source::MacroExpansionFact;

use super::review::{MacroBindingResult, review, review_without_definitions};

pub(crate) fn binds_allowance(
    expansion: &MacroExpansionFact,
    allowance: &MacroExpansionAllow,
) -> bool {
    let allowed = BTreeMap::from([(allowance.name.as_str(), allowance)]);
    matches!(
        review_without_definitions(expansion, &allowed),
        MacroBindingResult::Bound { .. }
    )
}

pub(crate) fn closes_async_syntax(
    contract: &zrail_core::Contract,
    source: &crate::source::SourceIndex,
    resolved_cargo: Option<&crate::cargo::ResolvedCargoGraph>,
    expansion: &MacroExpansionFact,
) -> bool {
    if directly_inspected(expansion) {
        return true;
    }
    let allowed = contract
        .source
        .rust
        .macros
        .allow
        .iter()
        .filter(|allowed| allowed.async_syntax == MacroAsyncSyntax::None)
        .map(|allowed| (allowed.name.as_str(), allowed))
        .collect::<BTreeMap<_, _>>();
    matches!(
        review(source, resolved_cargo, expansion, &allowed),
        MacroBindingResult::Bound { .. }
    )
}

pub(crate) fn closes_type_duplication(
    contract: &zrail_core::Contract,
    source: &crate::source::SourceIndex,
    resolved_cargo: Option<&crate::cargo::ResolvedCargoGraph>,
    expansion: &MacroExpansionFact,
) -> bool {
    if expansion.is_compiler_builtin() {
        return true;
    }
    let allowed = contract
        .source
        .rust
        .macros
        .allow
        .iter()
        .filter(|allowed| allowed.duplication_effect == MacroDuplicationEffect::None)
        .map(|allowed| (allowed.name.as_str(), allowed))
        .collect::<BTreeMap<_, _>>();
    matches!(
        review(source, resolved_cargo, expansion, &allowed),
        MacroBindingResult::Bound {
            confidence: AnalysisQuality::Exact,
            ..
        }
    )
}

pub(super) fn directly_inspected(expansion: &MacroExpansionFact) -> bool {
    if expansion.quality != AnalysisQuality::Exact || !expansion.is_compiler_builtin() {
        return false;
    }
    expansion
        .candidates
        .iter()
        .flat_map(crate::source::MacroCandidate::policy_names)
        .all(|name| {
            let leaf = name.rsplit("::").next().unwrap_or(name);
            matches!(
                leaf,
                "cfg"
                    | "column"
                    | "concat"
                    | "concat_bytes"
                    | "env"
                    | "file"
                    | "include"
                    | "include_bytes"
                    | "include_str"
                    | "line"
                    | "module_path"
                    | "option_env"
                    | "stringify"
            )
        })
}
