//! Dependency identity attestations are exact, reasoned, and non-overlapping.

use std::collections::BTreeSet;

use super::{
    Contract, CrateRootSource,
    validate_limits::ValidationErrors,
    validate_paths::{
        validate_package_name, validate_repository_literal, validate_repository_pattern,
    },
    validate_sets::require_reason,
    validate_source::valid_rust_path,
};

pub(super) fn validate(contract: &Contract, errors: &mut ValidationErrors) {
    let mut identities = BTreeSet::new();
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
        validate_source(&attestation.source, &attestation.package, errors);
        if matches!(attestation.source, CrateRootSource::Repository { .. }) {
            errors.push(format!(
                "dependency crate-root attestation for {:?} may not select repository macro provenance",
                attestation.package
            ));
        }
        if let CrateRootSource::CargoLock { package, .. } = &attestation.source
            && package != &attestation.package
        {
            errors.push(format!(
                "dependency crate-root attestation for {:?} selects different Cargo.lock package {package:?}",
                attestation.package
            ));
        }
        if !identities.insert((attestation.package.as_str(), attestation.source.identity())) {
            errors.push(format!(
                "duplicate dependency crate-root attestation for {:?} at {}",
                attestation.package,
                attestation.source.identity()
            ));
        }
    }
}

pub(super) fn validate_source(
    source: &CrateRootSource,
    package: &str,
    errors: &mut ValidationErrors,
) {
    let invalid = match source {
        CrateRootSource::Legacy => false,
        CrateRootSource::CargoLock {
            package,
            version,
            source,
        } => {
            validate_package_name(package, errors);
            version
                .as_ref()
                .is_some_and(|value| value.trim().is_empty())
                || source.as_ref().is_some_and(|value| value.trim().is_empty())
        }
        CrateRootSource::Registry {
            registry,
            index,
            requirement,
        } => {
            (registry.is_some() && index.is_some())
                || requirement.trim().is_empty()
                || optional_empty(registry.as_ref())
                || optional_empty(index.as_ref())
        }
        CrateRootSource::Git {
            repository,
            branch,
            tag,
            rev,
            requirement,
        } => {
            repository.trim().is_empty()
                || [branch, tag, rev]
                    .into_iter()
                    .filter(|value| value.is_some())
                    .count()
                    > 1
                || optional_empty(branch.as_ref())
                || optional_empty(tag.as_ref())
                || optional_empty(rev.as_ref())
                || optional_empty(requirement.as_ref())
        }
        CrateRootSource::Repository {
            package,
            directory,
            inputs,
            ..
        } => {
            validate_package_name(package, errors);
            validate_repository_literal(directory, errors);
            if inputs.len() > 4_096 {
                errors.push("repository macro inputs exceed the 4096-pattern safety limit".into());
            }
            let mut seen = BTreeSet::new();
            for input in inputs {
                validate_repository_pattern(input, errors);
                if !seen.insert(input) {
                    errors.push(format!("duplicate repository macro input {input:?}"));
                }
                if input
                    .split('/')
                    .any(|part| matches!(part, ".git" | ".zrail" | "target" | "zrail.lock"))
                {
                    errors.push(format!(
                        "repository macro input {input:?} selects reserved metadata or build output"
                    ));
                }
            }
            package.trim().is_empty() || directory.trim().is_empty()
        }
    };
    if invalid {
        errors.push(format!(
            "dependency crate-root attestation for {package:?} has an invalid exact source identity"
        ));
    }
}

fn optional_empty(value: Option<&String>) -> bool {
    value.is_some_and(|value| value.trim().is_empty())
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
