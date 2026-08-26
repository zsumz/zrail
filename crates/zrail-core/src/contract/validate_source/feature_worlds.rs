//! Feature-world declarations are unique, complete-looking authored maps.

use std::collections::BTreeSet;

use crate::contract::{
    Contract, validate_limits::ValidationErrors, validate_paths::validate_package_name,
    validate_sets::require_reason,
};

pub(super) fn validate(contract: &Contract, errors: &mut ValidationErrors) {
    let mut worlds = BTreeSet::new();
    for world in &contract.source.rust.feature_worlds {
        if !valid_name(&world.name) {
            errors.push(format!(
                "Cargo feature world name {:?} is not a stable identifier",
                world.name
            ));
        } else if !worlds.insert(world.name.as_str()) {
            errors.push(format!("duplicate Cargo feature world {:?}", world.name));
        }
        require_reason("Cargo feature world", &world.name, &world.reason, errors);
        if world.packages.is_empty() {
            errors.push(format!(
                "Cargo feature world {:?} selects no packages",
                world.name
            ));
        }
        let mut packages = BTreeSet::new();
        for package in &world.packages {
            validate_package_name(&package.package, errors);
            if !packages.insert(package.package.as_str()) {
                errors.push(format!(
                    "Cargo feature world {:?} repeats package {:?}",
                    world.name, package.package
                ));
            }
            let mut features = BTreeSet::new();
            for feature in &package.features {
                if feature.trim().is_empty() {
                    errors.push(format!(
                        "Cargo feature world {:?} package {:?} contains an empty feature",
                        world.name, package.package
                    ));
                } else if !features.insert(feature.as_str()) {
                    errors.push(format!(
                        "Cargo feature world {:?} package {:?} repeats feature {:?}",
                        world.name, package.package, feature
                    ));
                }
            }
        }
    }
}

fn valid_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

#[cfg(test)]
#[path = "feature_worlds_test.rs"]
mod feature_worlds_test;
