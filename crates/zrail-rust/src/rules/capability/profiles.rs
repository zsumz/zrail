//! Effect profiles select package files and policy-applicable source facts.

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::{AnalysisQuality, FindingSink, PolicyReachability, glob_matches};

use crate::{
    cargo::Package,
    source::{ObservedFact, RustFileFacts},
};

use super::{super::RuleContext, effect_tokens, effects::finding, path_matches};

pub(super) fn check(context: &RuleContext<'_>, findings: &mut FindingSink) {
    let assignments = package_profiles(context);
    let mut emitted = BTreeSet::new();
    for file in &context.source.files {
        for profile_name in profiles_for_file(file, &context.cargo.packages, &assignments) {
            let Some(profile) = context.contract.profiles.get(profile_name) else {
                continue;
            };
            for effect in &profile.effects.deny {
                for token in effect_tokens(*effect) {
                    for path in file.paths.iter().filter(|path| {
                        applies(profile.reachability, file, path) && path_matches(token, path)
                    }) {
                        let key = (file.relative.clone(), path.span, profile_name, *effect);
                        if emitted.insert(key) {
                            findings.push(finding(file, path, profile_name, *effect));
                        }
                    }
                }
                for boundary in file.compile_effects.iter().filter(|boundary| {
                    boundary.effect == *effect
                        && boundary.invocation.quality == AnalysisQuality::Exact
                        && boundary.invocation.is_compiler_builtin()
                        && applies(profile.reachability, file, &boundary.invocation.observation)
                }) {
                    let key = (
                        file.relative.clone(),
                        boundary.invocation.span,
                        profile_name,
                        *effect,
                    );
                    if emitted.insert(key) {
                        findings.push(finding(
                            file,
                            &boundary.invocation.observation,
                            profile_name,
                            *effect,
                        ));
                    }
                }
            }
        }
    }
}

fn applies(policy: PolicyReachability, file: &RustFileFacts, fact: &ObservedFact) -> bool {
    policy == PolicyReachability::All || fact.is_production_applicable(file.reachability)
}

fn profiles_for_file<'a>(
    file: &RustFileFacts,
    packages: &'a [Package],
    assignments: &BTreeMap<&'a str, Vec<&'a str>>,
) -> BTreeSet<&'a str> {
    let mut profiles = BTreeSet::new();
    if file.packages.is_empty() {
        if let Some(package) = package_for_file(packages, &file.relative)
            && let Some(assigned) = assignments.get(package.name.as_str())
        {
            profiles.extend(assigned.iter().copied());
        }
        return profiles;
    }
    for package in &file.packages {
        if let Some(assigned) = assignments.get(package.as_str()) {
            profiles.extend(assigned.iter().copied());
        }
    }
    profiles
}

fn package_profiles<'a>(context: &'a RuleContext<'_>) -> BTreeMap<&'a str, Vec<&'a str>> {
    let mut result = BTreeMap::new();
    for package in &context.cargo.packages {
        let profiles = context
            .contract
            .layers
            .iter()
            .filter(|layer| {
                layer
                    .packages
                    .iter()
                    .any(|pattern| glob_matches(pattern, &package.name))
            })
            .flat_map(|layer| layer.profiles.iter().map(String::as_str))
            .collect::<Vec<_>>();
        result.insert(package.name.as_str(), profiles);
    }
    result
}

fn package_for_file<'a>(packages: &'a [Package], file: &str) -> Option<&'a Package> {
    packages
        .iter()
        .filter(|package| package.contains_file(file))
        .max_by_key(|package| package.directory.len())
}
