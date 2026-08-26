//! Dependency topology and effect-profile permission changes.

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    Contract, DependencyEdgeKind, DependencyReachability, DependencyRule, Effect,
    PolicyReachability,
};

use super::{ArchitectureChange, ChangeKind, support::compare_set_values, topology_policy};

pub(super) fn compare(before: &Contract, after: &Contract, changes: &mut Vec<ArchitectureChange>) {
    topology_policy::compare_dependency_modes(before, after, changes);
    topology_policy::compare_crate_roots(before, after, changes);
    compare_layer_edges(before, after, changes);
    topology_policy::compare_layer_profiles(before, after, changes);
    topology_policy::compare_layer_external_modes(before, after, changes);
    compare_package_layers(before, after, changes);
    compare_profiles(before, after, changes);
    compare_dependency_rules(before, after, changes);
}

fn compare_layer_edges(before: &Contract, after: &Contract, changes: &mut Vec<ArchitectureChange>) {
    compare_set_values(
        "dependency.layer-edge",
        &layer_edges(before),
        &layer_edges(after),
        ChangeKind::Grant,
        ChangeKind::Revoke,
        "permits a layer dependency",
        changes,
    );
}

fn compare_package_layers(
    before: &Contract,
    after: &Contract,
    changes: &mut Vec<ArchitectureChange>,
) {
    let old = package_layers(before);
    let new = package_layers(after);
    let packages = old
        .keys()
        .chain(new.keys())
        .copied()
        .collect::<BTreeSet<_>>();
    for package in packages {
        let left = old.get(package);
        let right = new.get(package);
        if left != right {
            changes.push(
                ArchitectureChange::new(
                    ChangeKind::Unknown,
                    "dependency.package-layer",
                    package,
                    "package layer assignment changed and requires review",
                )
                .values(
                    left.copied().unwrap_or("<unassigned>"),
                    right.copied().unwrap_or("<unassigned>"),
                ),
            );
        }
    }
}

fn compare_profiles(before: &Contract, after: &Contract, changes: &mut Vec<ArchitectureChange>) {
    let profiles = before
        .profiles
        .keys()
        .chain(after.profiles.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    for name in profiles {
        compare_profile_reachability(before, after, &name, changes);
        let old = denied_effects(before, &name);
        let new = denied_effects(after, &name);
        for effect in new.difference(&old) {
            changes.push(ArchitectureChange::new(
                ChangeKind::Revoke,
                "effect.boundary",
                format!("{name}:{effect:?}"),
                "profile now denies this architectural effect",
            ));
        }
        for effect in old.difference(&new) {
            changes.push(ArchitectureChange::new(
                ChangeKind::Grant,
                "effect.boundary",
                format!("{name}:{effect:?}"),
                "profile no longer denies this architectural effect",
            ));
        }
        let old = denied_syntax(before, &name);
        let new = denied_syntax(after, &name);
        for syntax in new.difference(&old) {
            changes.push(ArchitectureChange::new(
                ChangeKind::Revoke,
                "syntax.boundary",
                format!("{name}:{syntax:?}"),
                "profile now denies this runtime-neutral syntax",
            ));
        }
        for syntax in old.difference(&new) {
            changes.push(ArchitectureChange::new(
                ChangeKind::Grant,
                "syntax.boundary",
                format!("{name}:{syntax:?}"),
                "profile no longer denies this runtime-neutral syntax",
            ));
        }
    }
}

fn compare_profile_reachability(
    before: &Contract,
    after: &Contract,
    name: &str,
    changes: &mut Vec<ArchitectureChange>,
) {
    let old = before
        .profiles
        .get(name)
        .map(|profile| profile.reachability);
    let new = after.profiles.get(name).map(|profile| profile.reachability);
    let (Some(old), Some(new)) = (old, new) else {
        return;
    };
    let kind = match (old, new) {
        (PolicyReachability::All, PolicyReachability::Production) => Some(ChangeKind::Grant),
        (PolicyReachability::Production, PolicyReachability::All) => Some(ChangeKind::Revoke),
        _ => None,
    };
    if let Some(kind) = kind {
        changes.push(
            ArchitectureChange::new(
                kind,
                "effect.reachability",
                name,
                "profile source reachability changed",
            )
            .values(format!("{old:?}"), format!("{new:?}")),
        );
    }
}

fn compare_dependency_rules(
    before: &Contract,
    after: &Contract,
    changes: &mut Vec<ArchitectureChange>,
) {
    compare_set_values(
        "dependency.explicit-deny",
        &dependency_denials(&before.dependency_rules),
        &dependency_denials(&after.dependency_rules),
        ChangeKind::Revoke,
        ChangeKind::Grant,
        "denies resolved package reachability",
        changes,
    );
}

fn layer_edges(contract: &Contract) -> BTreeSet<String> {
    contract
        .layers
        .iter()
        .flat_map(|layer| {
            layer
                .may_depend_on
                .iter()
                .map(move |dependency| format!("{}->{dependency}", layer.name))
        })
        .collect()
}

fn package_layers(contract: &Contract) -> BTreeMap<&str, &str> {
    let mut result = BTreeMap::new();
    for layer in &contract.layers {
        for package in &layer.packages {
            result.insert(package.as_str(), layer.name.as_str());
        }
    }
    result
}

fn denied_effects(contract: &Contract, profile: &str) -> BTreeSet<Effect> {
    contract
        .profiles
        .get(profile)
        .map_or_else(BTreeSet::new, |value| {
            value.effects.deny.iter().copied().collect()
        })
}

fn denied_syntax(contract: &Contract, profile: &str) -> BTreeSet<crate::AsyncSyntax> {
    contract
        .profiles
        .get(profile)
        .map_or_else(BTreeSet::new, |value| {
            value.syntax.deny.iter().copied().collect()
        })
}

fn dependency_denials(rules: &[DependencyRule]) -> BTreeSet<String> {
    rules
        .iter()
        .flat_map(|rule| {
            let kinds = if rule.kinds.is_empty() {
                vec![
                    DependencyEdgeKind::Normal,
                    DependencyEdgeKind::Development,
                    DependencyEdgeKind::Build,
                ]
            } else {
                rule.kinds.clone()
            };
            let depths = match rule.reachability {
                DependencyReachability::Direct => vec!["direct"],
                DependencyReachability::Transitive => vec!["direct", "transitive"],
            };
            rule.deny.iter().flat_map(move |target| {
                let kinds = kinds.clone();
                depths.clone().into_iter().flat_map(move |depth| {
                    kinds
                        .clone()
                        .into_iter()
                        .map(move |kind| format!("{}->{target}:{depth}:{kind:?}", rule.from))
                })
            })
        })
        .collect()
}

#[cfg(test)]
#[path = "topology_test.rs"]
mod topology_test;
