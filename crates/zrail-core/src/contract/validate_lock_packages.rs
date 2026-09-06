//! Lock inventories use bounded exact identities and unique policy names.

use std::collections::BTreeSet;

use super::{
    Contract, LockPackageAssertion, validate_limits::ValidationErrors,
    validate_paths::validate_package_name, validate_sets::require_reason,
};

pub(super) fn validate(contract: &Contract, errors: &mut ValidationErrors) {
    let rules = &contract.dependencies.lock_packages;
    if rules.len() > 1_024 {
        errors.push("dependencies.lock_package exceeds the 1024-rule safety limit".into());
    }
    let mut names = BTreeSet::new();
    for rule in rules {
        if !literal(&rule.name, 256) || !names.insert(&rule.name) {
            errors.push(
                "lock package rules require unique nonempty names of at most 256 bytes".into(),
            );
        }
        validate_package_name(&rule.package, errors);
        require_reason("lock package inventory", &rule.name, &rule.reason, errors);
        match &rule.assertion {
            LockPackageAssertion::Count { count } => {
                if *count > 100_000 {
                    errors.push("lock package count exceeds the 100000-node safety limit".into());
                }
            }
            LockPackageAssertion::Exact { identities } => {
                if identities.len() > 1_024 {
                    errors.push("lock package exact inventory exceeds 1024 identities".into());
                }
                let mut identities_seen = BTreeSet::new();
                let mut bytes = 0;
                for identity in identities {
                    bytes += identity.version.len()
                        + identity.source.len()
                        + identity.checksum.as_ref().map_or(0, String::len);
                    if !literal(&identity.version, 1_024) || !literal(&identity.source, 4_096) {
                        errors.push("lock identities require literal version/source strings of at most 1024/4096 bytes".into());
                    }
                    if identity.checksum.as_ref().is_some_and(|checksum| {
                        checksum.len() != 64
                            || !checksum
                                .bytes()
                                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
                    }) {
                        errors.push("lock identity checksum must be a lowercase SHA-256".into());
                    }
                    if !identities_seen.insert(identity) {
                        errors.push("lock package inventory contains duplicate identities".into());
                    }
                }
                if bytes > 16 * 1_024 {
                    errors.push(
                        "lock package inventory exceeds 16384 expected identity bytes".into(),
                    );
                }
            }
        }
    }
}

fn literal(value: &str, maximum: usize) -> bool {
    !value.is_empty()
        && value.len() <= maximum
        && value.trim() == value
        && !value.chars().any(char::is_control)
}

pub(super) fn item_count(contract: &Contract) -> usize {
    contract
        .dependencies
        .lock_packages
        .iter()
        .map(|rule| {
            1 + match &rule.assertion {
                LockPackageAssertion::Count { .. } => 1,
                LockPackageAssertion::Exact { identities } => identities.len(),
            }
        })
        .sum()
}

#[cfg(test)]
#[path = "validate_lock_packages_test.rs"]
mod validate_lock_packages_test;
