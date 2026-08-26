//! Exact type policies are named, bounded, and non-ambiguous.

use std::collections::BTreeSet;

use crate::contract::{
    CloneCopyPolicy, Contract, RustFieldContract, RustTypeKind, validate_limits::ValidationErrors,
    validate_paths::validate_repository_literal, validate_sets::require_reason,
};

use super::{type_identity::valid_exact_type, valid_rust_path};

pub(super) fn validate(contract: &Contract, errors: &mut ValidationErrors) {
    unique_traits(
        "source.rust.duplication.deny_imports",
        &contract.source.rust.duplication.deny_imports,
        errors,
    );
    unique_traits(
        "source.rust.duplication.deny_macro_tokens",
        &contract.source.rust.duplication.deny_macro_tokens,
        errors,
    );
    let mut names = BTreeSet::new();
    let mut subjects = BTreeSet::new();
    for policy in &contract.source.rust.types {
        require_reason("Rust type policy", &policy.name, &policy.reason, errors);
        if policy.name.trim().is_empty() {
            errors.push("Rust type policy names may not be empty".into());
        } else if !names.insert(policy.name.as_str()) {
            errors.push(format!("duplicate Rust type policy {:?}", policy.name));
        }
        validate_repository_literal(&policy.path, errors);
        if !std::path::Path::new(&policy.path)
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("rs"))
        {
            errors.push(format!(
                "Rust type policy {:?} path must name a .rs source",
                policy.name
            ));
        }
        if !valid_rust_path(&policy.identity) || !policy.identity.contains("::") {
            errors.push(format!(
                "Rust type policy {:?} match must be a qualified Rust path: {:?}",
                policy.name, policy.identity
            ));
        }
        if !subjects.insert((policy.path.as_str(), policy.identity.as_str())) {
            errors.push(format!(
                "duplicate Rust type subject {:?} in {:?}",
                policy.identity, policy.path
            ));
        }
        if policy.deny.iter().collect::<BTreeSet<_>>().len() != policy.deny.len() {
            errors.push(format!(
                "Rust type policy {:?} contains a duplicate prohibition",
                policy.name
            ));
        }
        if let Some(visibility) = &policy.visibility {
            validate_visibility(&policy.name, visibility, errors);
        }
        if let Some(fields) = &policy.fields {
            validate_fields(&policy.name, fields, errors);
        }
        validate_authority(policy, errors);
        if policy.kind == RustTypeKind::Type
            && policy.deny.is_empty()
            && policy.clone_copy == CloneCopyPolicy::Allow
            && policy.visibility.is_none()
            && policy.leaf_module.is_none()
            && policy.fields.is_none()
        {
            errors.push(format!(
                "Rust type policy {:?} enables no shape or duplication rail",
                policy.name
            ));
        }
    }
}

fn unique_traits(label: &str, traits: &[crate::DuplicationTrait], errors: &mut ValidationErrors) {
    if traits.iter().collect::<BTreeSet<_>>().len() != traits.len() {
        errors.push(format!("{label} contains a duplicate trait"));
    }
}

fn validate_fields(name: &str, fields: &[RustFieldContract], errors: &mut ValidationErrors) {
    let mut names = BTreeSet::new();
    for field in fields {
        if !valid_identifier(&field.name) {
            errors.push(format!(
                "Rust type policy {name:?} has invalid field name {:?}",
                field.name
            ));
        } else if !names.insert(field.name.as_str()) {
            errors.push(format!(
                "Rust type policy {name:?} contains duplicate field {:?}",
                field.name
            ));
        }
        if !valid_exact_type(&field.type_identity) {
            errors.push(format!(
                "Rust type policy {name:?} field {:?} type must use a supported exact Rust type with qualified non-primitive paths: {:?}",
                field.name, field.type_identity
            ));
        }
        validate_visibility(name, &field.visibility, errors);
    }
}

fn validate_authority(policy: &crate::RustTypeContract, errors: &mut ValidationErrors) {
    if policy.kind != RustTypeKind::AuthorityToken {
        return;
    }
    if policy.clone_copy != CloneCopyPolicy::Forbidden {
        errors.push(format!(
            "authority-token policy {:?} requires clone_copy = \"forbidden\"",
            policy.name
        ));
    }
    if policy.visibility.as_deref() != Some("private") {
        errors.push(format!(
            "authority-token policy {:?} requires visibility = \"private\"",
            policy.name
        ));
    }
    if policy.leaf_module != Some(true) {
        errors.push(format!(
            "authority-token policy {:?} requires leaf_module = true",
            policy.name
        ));
    }
    let Some(fields) = &policy.fields else {
        errors.push(format!(
            "authority-token policy {:?} requires an exact fields array",
            policy.name
        ));
        return;
    };
    if fields.iter().any(|field| field.visibility != "private") {
        errors.push(format!(
            "authority-token policy {:?} requires every field to be private",
            policy.name
        ));
    }
}

fn validate_visibility(name: &str, visibility: &str, errors: &mut ValidationErrors) {
    if !valid_visibility(visibility) {
        errors.push(format!(
            "Rust type policy {name:?} has invalid semantic visibility {visibility:?}"
        ));
    }
}

fn valid_visibility(value: &str) -> bool {
    matches!(
        value,
        "private" | "pub" | "pub(crate)" | "pub(self)" | "pub(super)"
    ) || value
        .strip_prefix("pub(in ")
        .and_then(|value| value.strip_suffix(')'))
        .is_some_and(valid_rust_path)
}

fn valid_identifier(value: &str) -> bool {
    valid_rust_path(value) && !value.contains("::")
}

#[cfg(test)]
#[path = "types_test.rs"]
mod types_test;
