//! Effect-profile reachability permission classification.

use crate::{
    ChangeKind, DependencyEdgeKind, DependencyReachability, DependencyRule, Effect, EffectBoundary,
    PolicyReachability, ProfileContract, compare_architecture,
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
            syntax: crate::SyntaxBoundary::default(),
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

#[test]
fn expanding_a_direct_denial_to_transitive_is_only_a_revocation() {
    let mut direct = contract_with_hard_limit(300);
    direct.dependency_rules.push(DependencyRule {
        name: "runtime-boundary".into(),
        from: "app".into(),
        deny: vec!["blocked".into()],
        reachability: DependencyReachability::Direct,
        kinds: vec![DependencyEdgeKind::Normal],
        reason: "block the runtime path".into(),
    });
    let mut transitive = direct.clone();
    transitive.dependency_rules[0].reachability = DependencyReachability::Transitive;

    let expanded = compare_architecture(&direct, None, &transitive, None);

    assert!(expanded.changes.iter().any(|change| {
        change.kind == ChangeKind::Revoke && change.rail == "dependency.explicit-deny"
    }));
    assert!(!expanded.changes.iter().any(|change| {
        change.kind == ChangeKind::Grant && change.rail == "dependency.explicit-deny"
    }));
}
