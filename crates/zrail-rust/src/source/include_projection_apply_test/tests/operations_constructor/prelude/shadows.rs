//! Prelude lookup obeys Rust lexical, generic, and extern namespace precedence.

use super::super::canonicalize_operations_with_external;
use super::*;

#[test]
fn generic_option_shadows_the_implicit_prelude() {
    let mut index = index([parsed_file(
        "src/lib.rs",
        "fn marker() {} use self::marker as witness; fn identity<Option>(value: Option) -> Option { witness(); value }",
    )]);
    let findings = canonicalize_operations(&mut index, &domain(), &[]);

    assert!(
        findings
            .iter()
            .any(|finding| finding.id == "RUST-INCLUDE-002")
    );
    assert!(has_call(&index, "marker"));
    assert!(!has_call(&index, "self::marker"));
    assert!(!has_path(&index, "core::option::Option"));
}

#[test]
fn parameter_and_local_drop_shadow_the_implicit_prelude() {
    let index = canonicalized(
        "fn marker() {} use self::marker as witness; fn parameter(drop: fn(u8)) { witness(); drop(1); } fn local() { let drop = |_: u8| {}; drop(1); }",
    );

    assert!(has_call(&index, "marker"));
    assert!(!has_call(&index, "self::marker"));
    assert!(has_call(&index, "drop"));
    assert!(!has_call(&index, "core::mem::drop"));
}

#[test]
fn extern_type_root_precedes_the_type_prelude() {
    let mut index = index([parsed_file(
        "src/lib.rs",
        "fn make() { let _ = Vec::new(); }",
    )]);
    let findings = canonicalize_operations_with_external(&mut index, &domain(), &[], "Vec");

    assert!(findings.is_empty(), "unexpected findings: {findings:#?}");
    assert!(has_call(&index, "Vec::new"));
    assert!(!has_call(&index, "std::vec::Vec::new"));
}

#[test]
fn extern_value_root_does_not_precede_the_value_prelude() {
    let mut index = index([parsed_file("src/lib.rs", "fn run() { drop(1_u8); }")]);
    let findings = canonicalize_operations_with_external(&mut index, &domain(), &[], "drop");

    assert!(findings.is_empty(), "unexpected findings: {findings:#?}");
    assert!(has_call(&index, "core::mem::drop"));
}
