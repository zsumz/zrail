//! Repeated macro names require distinct provenance rather than global uniqueness.

use crate::{
    CrateRootSource, MacroBindingMode, MacroExpansionAllow, MacroExpansionBindings,
    MacroExpansionMode, MacroInputMode,
    contract::{validate::validate_contract, validate_fixture_test::minimal_contract},
};

#[test]
fn one_macro_name_may_bind_distinct_sources() {
    let mut contract = minimal_contract();
    contract.source.rust.macros.mode = MacroExpansionMode::DenyUnreviewed;
    contract.source.rust.macros.allow = vec![allowance("=1.0.0"), allowance("=2.0.0")];

    validate_contract(&contract).expect("distinct provenance may share one macro name");
}

#[test]
fn repeated_name_and_provenance_remains_invalid() {
    let mut contract = minimal_contract();
    contract.source.rust.macros.mode = MacroExpansionMode::DenyUnreviewed;
    contract.source.rust.macros.allow = vec![allowance("=1.0.0"), allowance("=1.0.0")];

    let error = validate_contract(&contract).expect_err("duplicate authority must fail");
    assert!(
        error
            .to_string()
            .contains("repeated names require distinct source or definition provenance")
    );
}

fn allowance(requirement: &str) -> MacroExpansionAllow {
    MacroExpansionAllow {
        name: "derive::Model".into(),
        inputs: MacroInputMode::Inspect,
        binding: MacroBindingMode::Exact,
        bindings: MacroExpansionBindings::Opaque,
        async_syntax: crate::MacroAsyncSyntax::Opaque,
        duplication_effect: crate::MacroDuplicationEffect::Opaque,
        source_operations: crate::MacroSourceOperations::Opaque,
        field_mutation: crate::MacroFieldMutation::Opaque,
        definition: None,
        source: Some(CrateRootSource::Registry {
            registry: None,
            index: None,
            requirement: requirement.into(),
        }),
        reason: "The exact implementation source was reviewed.".into(),
    }
}
