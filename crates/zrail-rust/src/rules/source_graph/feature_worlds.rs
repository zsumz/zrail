//! Exact feature worlds gate Cargo targets with `required-features`.

use std::collections::BTreeSet;

pub(super) fn target_enabled(
    target: &crate::cargo::CargoTarget,
    feature_world: Option<&str>,
    active_features: &BTreeSet<String>,
) -> bool {
    feature_world.is_none()
        || target
            .required_features
            .iter()
            .all(|required| active_features.contains(required))
}

#[cfg(test)]
#[path = "feature_worlds_test.rs"]
mod feature_worlds_test;
