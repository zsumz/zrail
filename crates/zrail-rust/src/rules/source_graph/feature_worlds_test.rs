//! Required target features select exact-world roots without narrowing legacy analysis.

use std::collections::BTreeSet;

use super::target_enabled;
use crate::cargo::{CargoTarget, CargoTargetKind};

#[test]
fn exact_world_seeds_only_targets_whose_required_features_are_active() {
    let target = target();
    let active = BTreeSet::from(["cli".into(), "strict".into()]);
    let inactive = BTreeSet::from(["cli".into()]);

    assert!(target_enabled(&target, Some("shipping"), &active));
    assert!(!target_enabled(&target, Some("minimal"), &inactive));
}

#[test]
fn legacy_conditional_mode_retains_required_feature_targets() {
    assert!(target_enabled(&target(), None, &BTreeSet::new()));
}

fn target() -> CargoTarget {
    CargoTarget {
        name: "tool".into(),
        path: "src/bin/tool.rs".into(),
        kind: CargoTargetKind::Binary,
        required_features: vec!["cli".into(), "strict".into()],
    }
}
