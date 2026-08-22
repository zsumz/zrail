//! Source-owner contracts bind exact Rust paths to bounded source owners.

use crate::{Contract, OwnerContract, OwnerKind, PolicyReachability, path::glob_matches};

use super::{
    super::{
        validate_paths::{validate_repository_literal, validate_repository_pattern},
        validate_source::valid_rust_path,
    },
    ValidationErrors,
};

pub(super) fn validate_contract(contract: &Contract, errors: &mut ValidationErrors) {
    for owner in &contract.owners {
        if owner.selector.trim().is_empty() || owner.allow.is_empty() {
            errors.push(format!("owner {:?} requires match and allow", owner.name));
        }
        match owner.kind {
            OwnerKind::Call => validate_call(owner, errors),
            OwnerKind::Capability => validate_capability(owner, errors),
            OwnerKind::Directory => validate_directory(owner, errors),
        }
    }
}

fn validate_call(owner: &OwnerContract, errors: &mut ValidationErrors) {
    validate_source_owner(owner, "call", errors);
    if !owner.selector.contains("::") {
        errors.push(format!(
            "call owner match must be a qualified Rust path: {:?}",
            owner.selector
        ));
    }
}

fn validate_capability(owner: &OwnerContract, errors: &mut ValidationErrors) {
    validate_source_owner(owner, "capability", errors);
}

fn validate_source_owner(owner: &OwnerContract, kind: &str, errors: &mut ValidationErrors) {
    if owner.within.is_empty() {
        errors.push(format!(
            "{kind} owner {:?} requires at least one within pattern",
            owner.name,
        ));
    }
    if !valid_rust_path(&owner.selector) {
        errors.push(format!(
            "{kind} owner match must be a Rust path: {:?}",
            owner.selector,
        ));
    }
    for pattern in &owner.within {
        validate_repository_pattern(pattern, errors);
    }
    for path in &owner.allow {
        validate_repository_literal(path, errors);
        if !owner
            .within
            .iter()
            .any(|pattern| glob_matches(pattern, path))
        {
            errors.push(format!(
                "{kind} owner {:?} allows a path outside its within patterns",
                owner.name,
            ));
        }
    }
}

fn validate_directory(owner: &OwnerContract, errors: &mut ValidationErrors) {
    if owner.reachability != PolicyReachability::All {
        errors.push(format!(
            "directory owner {:?} requires reachability = \"all\"",
            owner.name
        ));
    }
    if !owner.within.is_empty() {
        errors.push(format!(
            "directory owner {:?} may not declare within",
            owner.name
        ));
    }
    validate_repository_pattern(&owner.selector, errors);
    for path in &owner.allow {
        validate_repository_literal(path, errors);
    }
    if !owner
        .allow
        .iter()
        .all(|path| glob_matches(&owner.selector, path))
    {
        errors.push(format!(
            "owner {:?} allows a path outside its selector",
            owner.name
        ));
    }
}

#[cfg(test)]
#[path = "owner_test.rs"]
mod owner_test;
