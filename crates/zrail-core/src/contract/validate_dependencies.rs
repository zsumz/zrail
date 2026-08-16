//! Dependency identity attestations are exact, reasoned, and non-overlapping.

use std::collections::BTreeSet;

use super::{
    Contract, validate_limits::ValidationErrors, validate_paths::validate_package_name,
    validate_sets::require_reason, validate_source::valid_rust_path,
};

pub(super) fn validate(contract: &Contract, errors: &mut ValidationErrors) {
    let mut packages = BTreeSet::new();
    for attestation in &contract.dependencies.crate_roots {
        validate_package_name(&attestation.package, errors);
        require_reason(
            "dependency crate-root attestation",
            &attestation.package,
            &attestation.reason,
            errors,
        );
        if !valid_crate_root(&attestation.root) {
            errors.push(format!(
                "dependency crate root must be one normalized Rust crate identifier: {:?}",
                attestation.root
            ));
        }
        if !packages.insert(attestation.package.as_str()) {
            errors.push(format!(
                "duplicate dependency crate-root attestation for {:?}",
                attestation.package
            ));
        }
    }
}

fn valid_crate_root(root: &str) -> bool {
    !root.starts_with("r#")
        && valid_rust_path(root)
        && !root.contains("::")
        && !matches!(root, "_" | "Self" | "crate" | "self" | "super")
}

#[cfg(test)]
#[path = "validate_dependencies_test.rs"]
mod validate_dependencies_test;
