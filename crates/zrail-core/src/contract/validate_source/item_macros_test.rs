//! Item-macro selector and provenance validation examples.

use crate::{CrateRootSource, ItemMacroContract, MacroBindingMode};

use crate::contract::{validate::validate_contract, validate_fixture_test::minimal_contract};

#[test]
fn path_and_within_are_mutually_exclusive() {
    let mut contract = minimal_contract();
    contract.source.rust.item_macros.push(allowance());

    let error = validate_contract(&contract).expect_err("mixed selectors must fail");

    assert!(
        error
            .to_string()
            .contains("may not combine path and within")
    );
}

#[test]
fn external_source_requires_explicit_exact_binding() {
    let mut contract = minimal_contract();
    let mut item_macro = allowance();
    item_macro.path = None;
    item_macro.source = Some(CrateRootSource::Registry {
        registry: None,
        index: None,
        requirement: "1".into(),
    });
    contract.source.rust.item_macros.push(item_macro.clone());

    let error = validate_contract(&contract).expect_err("implicit binding must fail");
    assert!(
        error
            .to_string()
            .contains("requires resolution = \"exact\"")
    );

    contract.source.rust.item_macros[0].binding = Some(MacroBindingMode::Exact);
    validate_contract(&contract).expect("exact source binding is valid");
}

fn allowance() -> ItemMacroContract {
    ItemMacroContract {
        name: "items".into(),
        path: Some("src/lib.rs".into()),
        within: vec!["src/**".into()],
        binding: None,
        source: None,
        manifest: None,
        reason: "Reviewed item boundary.".into(),
    }
}
