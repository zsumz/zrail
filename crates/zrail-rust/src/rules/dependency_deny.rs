//! Exact package-to-package dependency denials.

use zrail_core::{DependencyRule, Finding, FindingSink};

use crate::cargo::{DependencyKind, Package};

use super::RuleContext;

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
        check_package(package, rule, findings);
    }
}

fn check_package(package: &Package, rule: &DependencyRule, findings: &mut FindingSink) {
    for dependency in &package.dependencies {
        if rule.deny.iter().any(|denied| denied == &dependency.name) {
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

const fn dependency_kind(kind: DependencyKind) -> &'static str {
    match kind {
        DependencyKind::Normal => "normal",
        DependencyKind::Development => "development",
        DependencyKind::Build => "build",
    }
}
