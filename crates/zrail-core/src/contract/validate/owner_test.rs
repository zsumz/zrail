//! Source-owner validation keeps capabilities and calls exact and bounded.

use crate::{OwnerContract, OwnerKind, PolicyReachability};

use super::{
    ValidationErrors, validate_call, validate_capability, validate_directory,
    validate_exact_operation, validate_field_mutation, validate_method_name,
    validate_mutating_method_scope,
};

#[test]
fn capability_owners_require_rust_paths_and_bounded_allowed_files() {
    let valid = owner();
    assert!(errors(&valid).is_empty());

    let mut invalid = owner();
    invalid.selector = "std:::fs".into();
    invalid.allow = vec!["crates/other/src/io.rs".into()];
    let errors = errors(&invalid).join("\n");

    assert!(errors.contains("must be a Rust path"), "{errors}");
    assert!(errors.contains("outside its within patterns"), "{errors}");
}

#[test]
fn capability_owners_require_a_source_scope() {
    let mut invalid = owner();
    invalid.within.clear();

    let errors = errors(&invalid).join("\n");

    assert!(errors.contains("requires at least one within pattern"));
}

#[test]
fn call_owners_require_a_qualified_rust_path() {
    let mut valid = owner();
    valid.kind = OwnerKind::Call;
    valid.selector = "std::fs::metadata".into();
    let mut errors = ValidationErrors::new();
    validate_call(&valid, &mut errors);
    assert!(errors.is_empty());

    valid.selector = "metadata".into();
    let mut errors = ValidationErrors::new();
    validate_call(&valid, &mut errors);
    assert!(
        errors
            .finish()
            .join("\n")
            .contains("must be a qualified Rust path")
    );
}

#[test]
fn directory_owners_reject_source_reachability() {
    let mut invalid = owner();
    invalid.kind = OwnerKind::Directory;
    invalid.reachability = PolicyReachability::Production;
    invalid.within.clear();
    invalid.selector = "crates/store/**".into();
    let mut errors = ValidationErrors::new();

    validate_directory(&invalid, &mut errors);

    assert!(errors.finish().join("\n").contains("requires reachability"));
}

#[test]
fn exact_operation_owners_require_qualified_identities() {
    for kind in [
        OwnerKind::TypeConstruction,
        OwnerKind::FieldRead,
        OwnerKind::FieldWrite,
        OwnerKind::FieldMutableBorrow,
        OwnerKind::FieldMutation,
        OwnerKind::FieldAuthority,
    ] {
        let mut invalid = owner();
        invalid.kind = kind;
        invalid.selector = "State".into();

        let errors = contract_errors(&invalid).join("\n");

        assert!(errors.contains("qualified Rust path"), "{kind:?}: {errors}");
    }
}

#[test]
fn method_name_owners_reject_resolved_method_claims() {
    let mut valid = owner();
    valid.kind = OwnerKind::MethodName;
    valid.selector = "advance".into();
    assert!(contract_errors(&valid).is_empty());

    valid.selector = "crate::State::advance".into();
    let errors = contract_errors(&valid).join("\n");

    assert!(errors.contains("one written method name"), "{errors}");
}

#[test]
fn field_mutation_owners_require_canonical_written_methods() {
    let mut valid = owner();
    valid.kind = OwnerKind::FieldMutation;
    valid.selector = "crate::State::values".into();
    valid.mutating_methods = vec!["clear".into(), "push".into()];
    assert!(contract_errors(&valid).is_empty());

    valid.mutating_methods = vec!["push".into(), "clear".into(), "clear".into()];
    let errors = contract_errors(&valid).join("\n");
    assert!(errors.contains("sorted and unique"), "{errors}");

    valid.mutating_methods = vec!["Vec::push".into()];
    let errors = contract_errors(&valid).join("\n");
    assert!(errors.contains("simple Rust identifiers"), "{errors}");

    for invalid in [
        "_", "self", "crate", "fn", "await", "r#self", "r#_", "r#r#push",
    ] {
        valid.mutating_methods = vec![invalid.into()];
        let errors = contract_errors(&valid).join("\n");
        assert!(
            errors.contains("simple Rust identifiers"),
            "{invalid}: {errors}"
        );
    }

    for raw in ["r#type", "r#async", "r#gen", "r#ordinary"] {
        valid.mutating_methods = vec![raw.into()];
        assert!(contract_errors(&valid).is_empty(), "{raw}");
    }
}

#[test]
fn other_owner_kinds_reject_mutating_methods() {
    let mut invalid = owner();
    invalid.mutating_methods = vec!["push".into()];
    let mut errors = ValidationErrors::new();

    validate_mutating_method_scope(&invalid, &mut errors);

    assert!(
        errors
            .finish()
            .join("\n")
            .contains("may not declare mutating_methods")
    );
}

fn errors(owner: &OwnerContract) -> Vec<String> {
    let mut errors = ValidationErrors::new();
    validate_capability(owner, &mut errors);
    errors.finish()
}

fn contract_errors(owner: &OwnerContract) -> Vec<String> {
    let mut errors = ValidationErrors::new();
    match owner.kind {
        OwnerKind::TypeConstruction => {
            validate_exact_operation(owner, "type-construction", &mut errors);
        }
        OwnerKind::FieldRead => validate_exact_operation(owner, "field-read", &mut errors),
        OwnerKind::FieldWrite => validate_exact_operation(owner, "field-write", &mut errors),
        OwnerKind::FieldMutableBorrow => {
            validate_exact_operation(owner, "field-mutable-borrow", &mut errors);
        }
        OwnerKind::FieldMutation => validate_field_mutation(owner, &mut errors),
        OwnerKind::FieldAuthority => {
            validate_exact_operation(owner, "field-authority", &mut errors);
        }
        OwnerKind::MethodName => validate_method_name(owner, &mut errors),
        kind => errors.push(format!("unexpected operation owner fixture kind: {kind:?}")),
    }
    errors.finish()
}

fn owner() -> OwnerContract {
    OwnerContract {
        name: "filesystem".into(),
        kind: OwnerKind::Capability,
        reachability: PolicyReachability::All,
        within: vec!["crates/store/src/**".into()],
        selector: "std::fs".into(),
        mutating_methods: Vec::new(),
        allow: vec!["crates/store/src/io.rs".into()],
        reason: "one filesystem owner".into(),
    }
}
