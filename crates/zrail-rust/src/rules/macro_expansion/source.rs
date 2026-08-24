//! Expansion authority is bound to observed compiler, repository, or dependency origin.

use zrail_core::{CrateRootSource, MacroExpansionAllow};

use crate::{
    cargo::{ResolvedCargoGraph, source_matches},
    source::{MacroCandidate, MacroOrigin},
};

use super::failure::MacroBindingFailure;

pub(super) fn failures(
    candidate: &MacroCandidate,
    allowance: &MacroExpansionAllow,
    resolved_cargo: Option<&ResolvedCargoGraph>,
) -> Vec<MacroBindingFailure> {
    let mut failures = candidate
        .origins
        .iter()
        .filter_map(|origin| mismatch(candidate, allowance, origin, resolved_cargo))
        .collect::<Vec<_>>();
    failures.sort();
    failures.dedup();
    failures
}

fn mismatch(
    candidate: &MacroCandidate,
    allowance: &MacroExpansionAllow,
    origin: &MacroOrigin,
    resolved_cargo: Option<&ResolvedCargoGraph>,
) -> Option<MacroBindingFailure> {
    match origin {
        MacroOrigin::CompilerBuiltin
            if allowance.source.is_some() || allowance.definition.is_some() =>
        {
            Some(source_mismatch(allowance, vec!["compiler".into()]))
        }
        MacroOrigin::Repository { package, directory } if allowance.source.is_some() => Some(
            source_mismatch(allowance, vec![format!("repository:{package}:{directory}")]),
        ),
        MacroOrigin::Repository { .. }
            if allowance.definition.is_none()
                && !candidate.policy_names().all(|name| name.contains("::")) =>
        {
            Some(MacroBindingFailure::ConfidenceNotGranted {
                allowance: allowance.name.clone(),
            })
        }
        MacroOrigin::External { package, source }
            if allowance.source.as_ref().is_none_or(|allowed| {
                !matches_external(allowed, package, source, resolved_cargo)
            }) =>
        {
            Some(source_mismatch(
                allowance,
                vec![format!("{package}@{}", source.identity())],
            ))
        }
        _ => None,
    }
}

fn matches_external(
    allowed: &CrateRootSource,
    package: &str,
    observed: &crate::cargo::DependencySource,
    resolved_cargo: Option<&ResolvedCargoGraph>,
) -> bool {
    let CrateRootSource::CargoLock {
        package: selected,
        version,
        source,
    } = allowed
    else {
        return source_matches(allowed, observed);
    };
    let Some(graph) = resolved_cargo else {
        return false;
    };
    let Ok(selected) = graph.lookup(selected, version.as_deref(), source.as_deref()) else {
        return false;
    };
    graph
        .package_for_source(package, observed)
        .is_ok_and(|observed| observed == selected)
}

fn source_mismatch(
    allowance: &MacroExpansionAllow,
    mut observed: Vec<String>,
) -> MacroBindingFailure {
    observed.sort();
    observed.dedup();
    MacroBindingFailure::SourceMismatch {
        allowance: allowance.name.clone(),
        expected: allowance.source.as_ref().map_or_else(
            || "explicit external source".into(),
            CrateRootSource::identity,
        ),
        observed,
    }
}

#[cfg(test)]
#[path = "source_test.rs"]
mod source_test;
