//! Expansion authority is bound to observed compiler, repository, or dependency origin.

use zrail_core::{CrateRootSource, MacroExpansionAllow};

use crate::{
    cargo::source_matches,
    source::{MacroCandidate, MacroOrigin},
};

use super::failure::MacroBindingFailure;

pub(super) fn failures(
    candidate: &MacroCandidate,
    allowance: &MacroExpansionAllow,
) -> Vec<MacroBindingFailure> {
    let mut failures = candidate
        .origins
        .iter()
        .filter_map(|origin| mismatch(candidate, allowance, origin))
        .collect::<Vec<_>>();
    failures.sort();
    failures.dedup();
    failures
}

fn mismatch(
    candidate: &MacroCandidate,
    allowance: &MacroExpansionAllow,
    origin: &MacroOrigin,
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
            if allowance
                .source
                .as_ref()
                .is_none_or(|allowed| !source_matches(allowed, source)) =>
        {
            Some(source_mismatch(
                allowance,
                vec![format!("{package}@{}", source.identity())],
            ))
        }
        _ => None,
    }
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
