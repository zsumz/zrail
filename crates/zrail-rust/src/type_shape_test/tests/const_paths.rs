//! Exact const paths resolve value authority, not a same-spelled type binding.

use super::{
    type_policy_test::{error_count, report, reset},
    type_shape_domain_test::{SHAPE, fixture},
};

#[test]
fn array_lengths_resolve_const_paths_in_the_value_namespace() {
    let root = fixture(
        &policy("[u8; crate::CAPACITY]"),
        "const CAPACITY: usize = 64; struct Permit { epoch: [u8; crate::CAPACITY] }",
    );
    let result = report(&root);
    assert_eq!(error_count(&result, "RUST-TYPE-002"), 0, "{result}");
    let coverage = super::governed_surface_report(&root, "zrail.toml".as_ref()).unwrap();
    assert!(coverage.type_policies[0].observations[0].allowed);
    reset(&root);
}

#[test]
fn braced_const_generic_paths_have_the_same_exact_value_identity() {
    let root = fixture(
        &policy("crate::Buffer<{crate::CAPACITY}>"),
        concat!(
            "const CAPACITY: usize = 64; struct Buffer<const N: usize> { bytes: [u8; N] }\n",
            "struct Permit { epoch: Buffer<{crate::CAPACITY}> }",
        ),
    );
    let result = report(&root);
    assert_eq!(error_count(&result, "RUST-TYPE-002"), 0, "{result}");
    reset(&root);
}

#[test]
fn a_const_alias_is_not_confused_with_a_type_of_the_same_name() {
    let root = fixture(
        &policy("[u8; crate::dimensions::CAPACITY]"),
        concat!(
            "mod dimensions { pub const CAPACITY: usize = 64; }\n",
            "type CAPACITY = u8; use crate::dimensions::CAPACITY;\n",
            "struct Permit { epoch: [u8; CAPACITY] }",
        ),
    );
    let result = report(&root);
    assert_eq!(error_count(&result, "RUST-TYPE-002"), 0, "{result}");
    reset(&root);
}

#[test]
fn missing_values_and_nontrivial_const_blocks_remain_unresolved() {
    for ty in ["[u8; crate::MISSING]", "Buffer<{ let size = 64; size }>"] {
        let root = fixture(
            &policy("[u8; crate::MISSING]"),
            &format!("struct Buffer<const N: usize>; struct Permit {{ epoch: {ty} }}"),
        );
        let result = report(&root);
        assert!(error_count(&result, "RUST-TYPE-002") > 0, "{result}");
        reset(&root);
    }
}

fn policy(ty: &str) -> String {
    SHAPE.replace("type = \"u64\"", &format!("type = {ty:?}"))
}
