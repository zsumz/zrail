//! Direct and transitive package denials use exact Cargo.lock paths when available.

use zrail_core::{DependencyReachability, DependencyRule, Finding, FindingSink};

use crate::cargo::Package;

use super::{
    RuleContext,
    dependency_paths::{dependency_kind, resolve_denied_paths, selected_kind},
};

pub(super) fn check_exact_denials(context: &RuleContext<'_>, findings: &mut FindingSink) {
    for rule in &context.contract.dependency_rules {
        let Some(package) = context
            .cargo
            .packages
            .iter()
            .find(|package| package.name == rule.from)
        else {
            findings.push(Finding::error(
                "DEP-007",
                &rule.name,
                "dependency",
                format!("dependency rule names missing package {:?}", rule.from),
            ));
            continue;
        };
        match (rule.reachability, context.resolved_cargo) {
            (DependencyReachability::Transitive, None) => missing_lock(package, rule, findings),
            (_, Some(graph)) => match resolve_denied_paths(package, rule, graph) {
                Ok(paths) => report_resolved(package, rule, &paths, findings),
                Err(error) => resolution_failure(package, rule, error, findings),
            },
            (DependencyReachability::Direct, None) => check_manifest(package, rule, findings),
        }
    }
}

fn check_manifest(package: &Package, rule: &DependencyRule, findings: &mut FindingSink) {
    for dependency in &package.dependencies {
        if selected_kind(rule, dependency.kind)
            && rule.deny.iter().any(|denied| denied == &dependency.name)
        {
            findings.push(
                Finding::error(
                    "DEP-008",
                    &rule.name,
                    "dependency",
                    format!(
                        "package {} has explicitly denied {} dependency {}",
                        package.name,
                        dependency_kind(dependency.kind),
                        dependency.name
                    ),
                )
                .at(package.manifest_path(), None)
                .because(&rule.reason),
            );
        }
    }
}

fn report_resolved(
    package: &Package,
    rule: &DependencyRule,
    paths: &[super::dependency_paths::ResolvedDependencyPath],
    findings: &mut FindingSink,
) {
    for path in paths {
        findings.push(
            Finding::error(
                "DEP-008",
                &rule.name,
                "dependency",
                format!(
                    "package {} has denied resolved {} dependency path: {}",
                    package.name,
                    dependency_kind(path.kind),
                    path.nodes
                        .iter()
                        .map(crate::cargo::ResolvedPackageIdentity::label)
                        .collect::<Vec<_>>()
                        .join(" -> ")
                ),
            )
            .at(package.manifest_path(), None)
            .because(&rule.reason),
        );
    }
}

fn missing_lock(package: &Package, rule: &DependencyRule, findings: &mut FindingSink) {
    resolution_failure(
        package,
        rule,
        "transitive dependency policy requires Cargo.lock".into(),
        findings,
    );
}

fn resolution_failure(
    package: &Package,
    rule: &DependencyRule,
    message: String,
    findings: &mut FindingSink,
) {
    findings.push(
        Finding::error("DEP-011", &rule.name, "dependency", message)
            .at(package.manifest_path(), None)
            .because(&rule.reason)
            .with_help(
                "regenerate Cargo.lock from reviewed manifests or narrow the ambiguous declaration",
            ),
    );
}
