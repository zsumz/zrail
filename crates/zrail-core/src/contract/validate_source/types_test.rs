//! Authority-token contracts require complete private Clone/Copy-closed shape.

use crate::{
    CloneCopyPolicy, RustFieldContract, RustTypeContract, RustTypeKind, TypeProhibition,
    contract::validate_fixture_test::minimal_contract,
};

use super::{ValidationErrors, validate};

#[test]
fn authority_token_requires_private_leaf_clone_copy_closed_exact_shape() {
    let mut contract = minimal_contract();
    contract.source.rust.types.push(RustTypeContract {
        name: "permit".into(),
        identity: "crate::authority::Permit".into(),
        path: "crates/app/src/authority.rs".into(),
        kind: RustTypeKind::AuthorityToken,
        reachability: crate::PolicyReachability::Production,
        deny: all_prohibitions(),
        clone_copy: CloneCopyPolicy::Allow,
        visibility: Some("pub(crate)".into()),
        leaf_module: Some(false),
        fields: None,
        reason: "Carries bounded authority.".into(),
    });

    let errors = errors(&contract);
    for expected in [
        "requires clone_copy = \"forbidden\"",
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
fn bundled_clone_copy_closure_rejects_redundant_explicit_prohibitions() {
    let mut contract = authority_contract("u64");
    let policy = &mut contract.source.rust.types[0];
    policy.kind = RustTypeKind::Type;
    policy.visibility = None;
    policy.leaf_module = None;
    policy.fields = None;
    policy.deny = vec![TypeProhibition::ImplClone];

    assert!(
        errors(&contract)
            .contains("cannot combine clone_copy = \"forbidden\" with explicit deny prohibitions")
    );
}

#[test]
fn complete_expanded_clone_copy_closure_is_valid_for_an_ordinary_type() {
    let mut contract = authority_contract("u64");
    let policy = &mut contract.source.rust.types[0];
    policy.kind = RustTypeKind::Type;
    policy.clone_copy = CloneCopyPolicy::Allow;
    policy.visibility = None;
    policy.leaf_module = None;
    policy.fields = None;
    policy.deny = all_prohibitions();

    assert_eq!(errors(&contract), "");
}

#[test]
fn exact_field_types_accept_complete_recursive_shapes() {
    for type_identity in [
        "core::option::Option<crate::authority::Permit>",
        "&'a mut crate::authority::Permit",
        "(u64,crate::authority::Permit)",
        "[crate::authority::Permit;crate::authority::COUNT]",
        "crate::Buffer<{crate::CAPACITY}>",
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
        "crate::Buffer<{let size = 64; size}>",
        "crate::Buffer<{crate::CAPACITY + 1}>",
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
        clone_copy: CloneCopyPolicy::Forbidden,
        visibility: Some("private".into()),
        leaf_module: Some(true),
        fields: Some(vec![RustFieldContract {
            name: "epoch".into(),
            type_identity: type_identity.into(),
            visibility: "private".into(),
        }]),
        reason: "Carries bounded authority.".into(),
    });
    contract
}

fn all_prohibitions() -> Vec<TypeProhibition> {
    vec![
        TypeProhibition::DeriveClone,
        TypeProhibition::DeriveCopy,
        TypeProhibition::ImplClone,
        TypeProhibition::ImplCopy,
        TypeProhibition::OpaqueExpansion,
    ]
}

fn errors(contract: &crate::Contract) -> String {
    let mut errors = ValidationErrors::new();
    validate(contract, &mut errors);
    errors.finish().join("\n")
}
