//! Staged effect claims do not close opaque expansion boundaries.

use zrail_core::MacroExpansionMode;

use super::super::binding_policy::{clean_allowance, contract, expansion, source};

#[test]
fn staged_source_operation_claim_is_inert() {
    let exact = expansion(
        "trusted::derive",
        "trusted::derive",
        zrail_core::AnalysisQuality::Exact,
        false,
    );
    let mut allowance = clean_allowance("trusted::derive");
    allowance.bindings = zrail_core::MacroExpansionBindings::Opaque;
    allowance.source_operations = zrail_core::MacroSourceOperations::None;
    let mut contract = contract(vec![allowance]);
    contract.source.rust.macros.mode = MacroExpansionMode::Allow;

    assert!(!crate::rules::closes_source_operations(
        &contract,
        &source(exact.clone()),
        None,
        &exact,
    ));
}
