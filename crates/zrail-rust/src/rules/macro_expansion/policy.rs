//! Closed macro-policy predicates shared by enforcement and coverage.

use zrail_core::{
    AnalysisQuality, MacroAsyncSyntax, MacroDuplicationEffect, MacroExpansionAllow,
    MacroExpansionMode, MacroFieldMutation, MacroSourceOperations, OwnerKind,
};

use crate::source::MacroExpansionFact;

use super::{
    allowances::AllowanceIndex,
    review::{MacroBindingResult, review, review_without_definitions},
};

pub(crate) fn binds_allowance(
    expansion: &MacroExpansionFact,
    allowance: &MacroExpansionAllow,
) -> bool {
    let allowed = AllowanceIndex::new([allowance]);
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
    if contract.source.rust.macros.mode == MacroExpansionMode::Allow {
        return false;
    }
    let allowed = contract
        .source
        .rust
        .macros
        .allow
        .iter()
        .filter(|allowed| {
            allowed.async_syntax == MacroAsyncSyntax::None && claim_provenance(expansion, allowed)
        })
        .collect::<Vec<_>>();
    let allowed = AllowanceIndex::new(allowed);
    matches!(
        review(source, resolved_cargo, expansion, &allowed),
        MacroBindingResult::Bound {
            confidence: AnalysisQuality::Exact,
            ..
        }
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
    if contract.source.rust.macros.mode == MacroExpansionMode::Allow {
        return false;
    }
    let allowed = contract
        .source
        .rust
        .macros
        .allow
        .iter()
        .filter(|allowed| {
            allowed.duplication_effect == MacroDuplicationEffect::None
                && claim_provenance(expansion, allowed)
        })
        .collect::<Vec<_>>();
    let allowed = AllowanceIndex::new(allowed);
    matches!(
        review(source, resolved_cargo, expansion, &allowed),
        MacroBindingResult::Bound {
            confidence: AnalysisQuality::Exact,
            ..
        }
    )
}

pub(crate) fn closes_source_operations(
    contract: &zrail_core::Contract,
    source: &crate::source::SourceIndex,
    resolved_cargo: Option<&crate::cargo::ResolvedCargoGraph>,
    expansion: &MacroExpansionFact,
) -> bool {
    if directly_inspected(expansion) {
        return true;
    }
    if contract.source.rust.macros.mode == MacroExpansionMode::Allow {
        return false;
    }
    let allowed = contract
        .source
        .rust
        .macros
        .allow
        .iter()
        .filter(|allowed| {
            allowed.source_operations == MacroSourceOperations::None
                && claim_provenance(expansion, allowed)
        })
        .collect::<Vec<_>>();
    let allowed = AllowanceIndex::new(allowed);
    matches!(
        review(source, resolved_cargo, expansion, &allowed),
        MacroBindingResult::Bound {
            confidence: AnalysisQuality::Exact,
            ..
        }
    )
}

pub(crate) fn closes_owned_operations(
    contract: &zrail_core::Contract,
    source: &crate::source::SourceIndex,
    resolved_cargo: Option<&crate::cargo::ResolvedCargoGraph>,
    expansion: &MacroExpansionFact,
    owner: OwnerKind,
) -> bool {
    if closes_source_operations(contract, source, resolved_cargo, expansion) {
        return true;
    }
    if !matches!(
        owner,
        OwnerKind::FieldWrite | OwnerKind::FieldMutableBorrow | OwnerKind::FieldMutation
    ) {
        return false;
    }
    if contract.source.rust.macros.mode == MacroExpansionMode::Allow {
        return false;
    }
    let allowed = contract
        .source
        .rust
        .macros
        .allow
        .iter()
        .filter(|allowed| {
            allowed.field_mutation == MacroFieldMutation::None
                && claim_provenance(expansion, allowed)
        })
        .collect::<Vec<_>>();
    let allowed = AllowanceIndex::new(allowed);
    matches!(
        review(source, resolved_cargo, expansion, &allowed),
        MacroBindingResult::Bound {
            confidence: AnalysisQuality::Exact,
            ..
        }
    )
}

fn claim_provenance(expansion: &MacroExpansionFact, allowance: &MacroExpansionAllow) -> bool {
    allowance.source.is_some() || allowance.definition.is_some() || expansion.is_compiler_builtin()
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

#[cfg(test)]
#[path = "policy_test.rs"]
mod policy_test;
