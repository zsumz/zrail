//! Authority-token contracts require complete private linear shape.

use crate::{
    RustFieldContract, RustTypeContract, RustTypeKind, TypeLinearity, TypeProhibition,
    contract::validate_fixture_test::minimal_contract,
};

use super::{ValidationErrors, validate};

#[test]
fn authority_token_requires_private_leaf_linear_exact_shape() {
    let mut contract = minimal_contract();
    contract.source.rust.types.push(RustTypeContract {
        name: "permit".into(),
        identity: "crate::authority::Permit".into(),
        path: "crates/app/src/authority.rs".into(),
        kind: RustTypeKind::AuthorityToken,
        reachability: crate::PolicyReachability::Production,
        deny: vec![TypeProhibition::ImplClone],
        linearity: TypeLinearity::Allow,
        visibility: Some("pub(crate)".into()),
        leaf_module: Some(false),
        fields: None,
        reason: "Carries one-use authority.".into(),
    });

    let errors = errors(&contract);
    for expected in [
        "requires linearity = \"required\"",
        "requires visibility = \"private\"",
        "requires leaf_module = true",
        "requires an exact fields array",
    ] {
        assert!(errors.contains(expected), "missing {expected:?}: {errors}");
    }
}

#[test]
fn exact_private_authority_shape_is_valid() {
    let contract = authority_contract("u64");

    assert_eq!(errors(&contract), "");
}

#[test]
fn exact_field_types_accept_complete_recursive_shapes() {
    for type_identity in [
        "core::option::Option<crate::authority::Permit>",
        "&'a mut crate::authority::Permit",
        "(u64,crate::authority::Permit)",
        "[crate::authority::Permit;crate::authority::COUNT]",
        "*const [u8]",
        "()",
        "!",
    ] {
        let errors = errors(&authority_contract(type_identity));
        assert!(
            errors.is_empty(),
            "unexpected rejection for {type_identity:?}: {errors}"
        );
    }
}

#[test]
fn exact_field_types_reject_lossy_or_unqualified_shapes() {
    for type_identity in [
        "impl crate::authority::Capability",
        "dyn crate::authority::Capability",
        "_",
        "crate::authority::make_type!()",
        "fn(crate::authority::Permit)",
        "core::iter::Iterator<Item = crate::authority::Permit>",
        "Option<crate::authority::Permit>",
        "core::option::Option<Permit>",
        "[u8;b'a']",
        "[u8;1.5]",
    ] {
        let errors = errors(&authority_contract(type_identity));
        assert!(
            errors.contains("supported exact Rust type"),
            "unexpected acceptance for {type_identity:?}: {errors}"
        );
    }
}

fn authority_contract(type_identity: &str) -> crate::Contract {
    let mut contract = minimal_contract();
    contract.source.rust.types.push(RustTypeContract {
        name: "permit".into(),
        identity: "crate::authority::Permit".into(),
        path: "crates/app/src/authority.rs".into(),
        kind: RustTypeKind::AuthorityToken,
        reachability: crate::PolicyReachability::Production,
        deny: Vec::new(),
        linearity: TypeLinearity::Required,
        visibility: Some("private".into()),
        leaf_module: Some(true),
        fields: Some(vec![RustFieldContract {
            name: "epoch".into(),
            type_identity: type_identity.into(),
            visibility: "private".into(),
        }]),
        reason: "Carries one-use authority.".into(),
    });
    contract
}

fn errors(contract: &crate::Contract) -> String {
    let mut errors = ValidationErrors::new();
    validate(contract, &mut errors);
    errors.finish().join("\n")
}
