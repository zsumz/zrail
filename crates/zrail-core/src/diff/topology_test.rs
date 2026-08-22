//! Effect-profile reachability permission classification.

use crate::{
    ChangeKind, Effect, EffectBoundary, PolicyReachability, ProfileContract, compare_architecture,
};

use crate::diff::compare_fixture_test::contract_with_hard_limit;

#[test]
fn narrowing_effect_evaluation_to_production_is_a_grant() {
    let mut all = contract_with_hard_limit(300);
    all.profiles.insert(
        "kernel".into(),
        ProfileContract {
            reachability: PolicyReachability::All,
            effects: EffectBoundary {
                deny: vec![Effect::Process],
            },
        },
    );
    let mut production = all.clone();
    production
        .profiles
        .get_mut("kernel")
        .expect("inserted profile")
        .reachability = PolicyReachability::Production;

    let narrowed = compare_architecture(&all, None, &production, None);
    let restored = compare_architecture(&production, None, &all, None);

    assert!(narrowed.changes.iter().any(|change| {
        change.kind == ChangeKind::Grant && change.rail == "effect.reachability"
    }));
    assert!(restored.changes.iter().any(|change| {
        change.kind == ChangeKind::Revoke && change.rail == "effect.reachability"
    }));
}
