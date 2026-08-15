//! Cargo workspace membership, layer direction, and exact edge denials.

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::{
    CycleMode, ExternalDependencyMode, Finding, FindingSink, PolicyMode, path::glob_matches,
};

use crate::cargo::{DependencyKind, Package};

use super::RuleContext;

pub(super) fn evaluate(context: &RuleContext<'_>, findings: &mut FindingSink) {
    check_workspace_members(context, findings);
    let assignments = layer_assignments(context, findings);
    check_edges(context, &assignments, findings);
    super::dependency_deny::check_exact_denials(context, findings);
    if context.contract.dependencies.cycles == CycleMode::Deny {
        super::dependency_cycle::check_cycles(context.cargo, findings);
    }
}

fn check_workspace_members(context: &RuleContext<'_>, findings: &mut FindingSink) {
    let declared = context
        .cargo
        .declared_members
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let observed = context
        .cargo
        .observed_members
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    for member in observed.difference(&declared) {
        findings.push(
            Finding::error(
                "DEP-001",
                "cargo.workspace-members",
                "dependency",
                format!("Cargo package {member:?} is absent from workspace.members"),
            )
            .at(format!("{member}/Cargo.toml"), None),
        );
    }
    for member in declared.difference(&observed) {
        findings.push(Finding::error(
            "DEP-002",
            "cargo.workspace-members",
            "dependency",
            format!("workspace.members names missing package directory {member:?}"),
        ));
    }
}

fn layer_assignments<'a>(
    context: &'a RuleContext<'_>,
    findings: &mut FindingSink,
) -> BTreeMap<&'a str, &'a str> {
    for layer in &context.contract.layers {
        for pattern in &layer.packages {
            if !context
                .cargo
                .packages
                .iter()
                .any(|package| glob_matches(pattern, &package.name))
            {
                findings.push(
                    Finding::error(
                        "DEP-010",
                        "dependency.package-layer",
                        "dependency",
                        format!(
                            "layer {:?} package selector {pattern:?} matches no Cargo package",
                            layer.name
                        ),
                    )
                    .because(&layer.reason)
                    .with_help("remove the stale selector or correct its package pattern"),
                );
            }
        }
    }
    let mut assignments = BTreeMap::new();
    for package in &context.cargo.packages {
        let matching = context
            .contract
            .layers
            .iter()
            .filter(|layer| {
                layer
                    .packages
                    .iter()
                    .any(|pattern| glob_matches(pattern, &package.name))
            })
            .collect::<Vec<_>>();
        match matching.as_slice() {
            [layer] => {
                assignments.insert(package.name.as_str(), layer.name.as_str());
            }
            [] if context.contract.dependencies.unassigned_packages == PolicyMode::Deny => {
                findings.push(
                    Finding::error(
                        "DEP-003",
                        "dependency.package-layer",
                        "dependency",
                        format!(
                            "package {:?} is not assigned to an architecture layer",
                            package.name
                        ),
                    )
                    .at(package_manifest(package), None),
                );
            }
            [] => {}
            _ => findings.push(
                Finding::error(
                    "DEP-004",
                    "dependency.package-layer",
                    "dependency",
                    format!(
                        "package {:?} matches multiple architecture layers",
                        package.name
                    ),
                )
                .at(package_manifest(package), None),
            ),
        }
    }
    assignments
}

fn check_edges(
    context: &RuleContext<'_>,
    assignments: &BTreeMap<&str, &str>,
    findings: &mut FindingSink,
) {
    for package in &context.cargo.packages {
        let Some(layer_name) = assignments.get(package.name.as_str()).copied() else {
            continue;
        };
        let Some(layer) = context
            .contract
            .layers
            .iter()
            .find(|layer| layer.name == layer_name)
        else {
            continue;
        };
        for dependency in &package.dependencies {
            if let Some(internal) = dependency.internal_package() {
                let Some(target_layer) = assignments.get(internal).copied() else {
                    continue;
                };
                let permitted = target_layer == layer.name
                    || layer.may_depend_on.iter().any(|name| name == target_layer);
                if !permitted {
                    findings.push(
                        Finding::error(
                            "DEP-005",
                            "dependency.layer-edge",
                            "dependency",
                            format!(
                                "package {} in layer {} may not depend on {} in layer {}",
                                package.name, layer.name, internal, target_layer
                            ),
                        )
                        .at(package_manifest(package), None)
                        .because(&layer.reason),
                    );
                }
            } else if layer.dependencies.external == ExternalDependencyMode::None {
                findings.push(
                    Finding::error(
                        "DEP-006",
                        "dependency.external",
                        "dependency",
                        format!(
                            "package {} in layer {} has {} external dependency {}",
                            package.name,
                            layer.name,
                            dependency_kind(dependency.kind),
                            dependency.alias
                        ),
                    )
                    .at(package_manifest(package), None)
                    .because(&layer.reason),
                );
            }
        }
    }
}

const fn dependency_kind(kind: DependencyKind) -> &'static str {
    match kind {
        DependencyKind::Normal => "normal",
        DependencyKind::Development => "development",
        DependencyKind::Build => "build",
    }
}

fn package_manifest(package: &Package) -> String {
    package.manifest_path()
}
