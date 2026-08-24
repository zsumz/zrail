//! Dependency coverage exposes the same exact shortest paths used by enforcement.

use zrail_core::{DependencyEdgeKind, DependencyReachability};

use crate::{
    cargo::ResolvedPackageIdentity,
    engine::RepositoryModel,
    rules::{dependency_kind, resolve_denied_paths},
};

use super::{GovernedDependencyPath, GovernedDependencyRule, GovernedPackageIdentity};

pub(super) fn report(model: &RepositoryModel) -> Result<Vec<GovernedDependencyRule>, String> {
    if model.bundle.contract.dependency_rules.is_empty() {
        return Ok(Vec::new());
    }
    let graph = model.resolved_cargo.as_ref().ok_or_else(|| {
        "coverage requires Cargo.lock to resolve dependency prohibitions exactly".to_owned()
    })?;
    let mut reports = Vec::new();
    for rule in &model.bundle.contract.dependency_rules {
        let package = model
            .cargo
            .packages
            .iter()
            .find(|package| package.name == rule.from)
            .ok_or_else(|| format!("dependency rule names missing package {:?}", rule.from))?;
        let mut deny = rule.deny.clone();
        deny.sort();
        deny.dedup();
        let mut paths = Vec::new();
        for path in resolve_denied_paths(package, rule, graph)? {
            let denied = path
                .nodes
                .last()
                .ok_or_else(|| "resolved dependency path has no destination".to_owned())?
                .name
                .clone();
            paths.push(GovernedDependencyPath {
                kind: dependency_kind(path.kind).into(),
                denied,
                nodes: path.nodes.iter().map(package_identity).collect(),
            });
        }
        paths.sort();
        reports.push(GovernedDependencyRule {
            policy_id: format!("dependency:{}", rule.name),
            name: rule.name.clone(),
            from: rule.from.clone(),
            deny,
            reachability: reachability(rule.reachability).into(),
            kinds: kinds(&rule.kinds),
            reason: rule.reason.clone(),
            paths,
        });
    }
    reports.sort_by(|left, right| left.policy_id.cmp(&right.policy_id));
    Ok(reports)
}

fn package_identity(identity: &ResolvedPackageIdentity) -> GovernedPackageIdentity {
    GovernedPackageIdentity {
        name: identity.name.clone(),
        version: identity.version.clone(),
        source: identity.source.clone(),
        checksum: identity.checksum.clone(),
    }
}

fn kinds(selected: &[DependencyEdgeKind]) -> Vec<String> {
    [
        DependencyEdgeKind::Normal,
        DependencyEdgeKind::Development,
        DependencyEdgeKind::Build,
    ]
    .into_iter()
    .filter(|kind| selected.is_empty() || selected.contains(kind))
    .map(|kind| edge_kind(kind).to_owned())
    .collect()
}

const fn edge_kind(kind: DependencyEdgeKind) -> &'static str {
    match kind {
        DependencyEdgeKind::Normal => "normal",
        DependencyEdgeKind::Development => "development",
        DependencyEdgeKind::Build => "build",
    }
}

const fn reachability(value: DependencyReachability) -> &'static str {
    match value {
        DependencyReachability::Direct => "direct",
        DependencyReachability::Transitive => "transitive",
    }
}
