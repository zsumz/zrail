//! Exact symbol scopes and semantic effect profiles.

mod compile;
mod effects;
mod ownership;
mod ownership_call;

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::{AnalysisQuality, Finding, FindingSink, path::glob_matches};

use crate::{
    cargo::Package,
    source::{ObservedFact, RustFileFacts},
};

use super::RuleContext;

use effects::finding as effect_finding;
pub(super) use effects::tokens as effect_tokens;

pub(super) fn evaluate(context: &RuleContext<'_>, findings: &mut FindingSink) {
    check_exact_scopes(context, findings);
    check_effect_profiles(context, findings);
    compile::check_paths(context, findings);
    ownership::check(context, findings);
}

fn check_exact_scopes(context: &RuleContext<'_>, findings: &mut FindingSink) {
    for scope in &context.contract.scopes {
        let mut stale = false;
        for pattern in &scope.include {
            if !context
                .source
                .files
                .iter()
                .any(|file| glob_matches(pattern, &file.relative))
            {
                stale = true;
                findings.push(
                    Finding::error(
                        "CAP-002",
                        &scope.name,
                        "capability",
                        format!("scope include {pattern:?} matches no Rust source"),
                    )
                    .because(&scope.reason),
                );
            }
        }
        for pattern in &scope.exclude {
            if !context
                .source
                .files
                .iter()
                .any(|file| glob_matches(pattern, &file.relative))
            {
                stale = true;
                findings.push(
                    Finding::error(
                        "CAP-003",
                        &scope.name,
                        "capability",
                        format!("scope exclusion {pattern:?} matches no Rust source"),
                    )
                    .because(&scope.reason),
                );
            }
        }
        if stale
            || !context
                .source
                .files
                .iter()
                .any(|file| matches_scope(&file.relative, &scope.include, &scope.exclude))
        {
            if !stale {
                findings.push(
                    Finding::error(
                        "CAP-002",
                        &scope.name,
                        "capability",
                        "capability scope matches no Rust source after exclusions",
                    )
                    .because(&scope.reason),
                );
            }
            continue;
        }
        for file in context
            .source
            .files
            .iter()
            .filter(|file| matches_scope(&file.relative, &scope.include, &scope.exclude))
        {
            for denied in &scope.symbols.deny {
                for path in file.paths.iter().filter(|path| path_matches(denied, path)) {
                    findings.push(
                        Finding::error(
                            "CAP-001",
                            &scope.name,
                            "capability",
                            format!("source reaches denied symbol {}", path.name),
                        )
                        .at(&file.relative, path.span)
                        .because(&scope.reason)
                        .with_analysis(path.quality)
                        .with_help("move the effect to its owning adapter and pass facts inward"),
                    );
                }
            }
        }
    }
}

fn check_effect_profiles(context: &RuleContext<'_>, findings: &mut FindingSink) {
    let assignments = package_profiles(context);
    let mut emitted = BTreeSet::new();
    for file in &context.source.files {
        let profiles = profiles_for_file(file, &context.cargo.packages, &assignments);
        for profile_name in profiles {
            let Some(profile) = context.contract.profiles.get(profile_name) else {
                continue;
            };
            for effect in &profile.effects.deny {
                for token in effect_tokens(*effect) {
                    for path in file.paths.iter().filter(|path| path_matches(token, path)) {
                        let key = (
                            file.relative.clone(),
                            path.span,
                            profile_name.to_string(),
                            *effect,
                        );
                        if emitted.insert(key) {
                            findings.push(effect_finding(file, path, profile_name, *effect));
                        }
                    }
                }
                for boundary in file.compile_effects.iter().filter(|boundary| {
                    boundary.effect == *effect
                        && boundary.invocation.quality == AnalysisQuality::Exact
                        && boundary.invocation.is_compiler_builtin()
                }) {
                    let key = (
                        file.relative.clone(),
                        boundary.invocation.span,
                        profile_name.to_string(),
                        *effect,
                    );
                    if emitted.insert(key) {
                        findings.push(effect_finding(
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

fn matches_scope(path: &str, include: &[String], exclude: &[String]) -> bool {
    include.iter().any(|pattern| glob_matches(pattern, path))
        && !exclude.iter().any(|pattern| glob_matches(pattern, path))
}

pub(super) fn path_matches(denied: &str, observed: &ObservedFact) -> bool {
    let denied = normalized_path(denied);
    observed.policy_names().any(|name| {
        let name = normalized_path(name);
        name == denied
            || name.starts_with(&format!("{denied}::"))
            || (observed.quality != AnalysisQuality::Exact
                && denied.starts_with(&format!("{name}::")))
    })
}

pub(super) fn normalized_path(path: &str) -> String {
    path.split("::")
        .map(|segment| segment.strip_prefix("r#").unwrap_or(segment))
        .collect::<Vec<_>>()
        .join("::")
}

#[cfg(test)]
#[path = "capability_test.rs"]
mod capability_test;
