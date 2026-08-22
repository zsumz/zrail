//! Source-owner validation keeps capabilities and calls exact and bounded.

use crate::{OwnerContract, OwnerKind, PolicyReachability};

use super::{ValidationErrors, validate_call, validate_capability, validate_directory};

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

fn errors(owner: &OwnerContract) -> Vec<String> {
    let mut errors = ValidationErrors::new();
    validate_capability(owner, &mut errors);
    errors.finish()
}

fn owner() -> OwnerContract {
    OwnerContract {
        name: "filesystem".into(),
        kind: OwnerKind::Capability,
        reachability: PolicyReachability::All,
        within: vec!["crates/store/src/**".into()],
        selector: "std::fs".into(),
        allow: vec!["crates/store/src/io.rs".into()],
        reason: "one filesystem owner".into(),
    }
}
