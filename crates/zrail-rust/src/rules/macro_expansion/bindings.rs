//! Optional `macro_rules!` definition hints must narrow the observed candidate.

use zrail_core::MacroExpansionAllow;

use crate::source::{MacroCandidate, MacroOrigin, SourceIndex};

use super::failure::MacroBindingFailure;

pub(super) fn failure(
    source: &SourceIndex,
    candidate: &MacroCandidate,
    allowance: &MacroExpansionAllow,
) -> Option<MacroBindingFailure> {
    let path = allowance.definition.as_deref()?;
    let bound_files = source
        .files
        .iter()
        .filter(|file| file.relative == path)
        .collect::<Vec<_>>();
    let definition_names = candidate
        .policy_names()
        .map(|name| name.rsplit("::").next().unwrap_or(name))
        .collect::<std::collections::BTreeSet<_>>();
    let bound = bound_files
        .iter()
        .flat_map(|file| &file.macro_definitions)
        .filter(|definition| definition_names.contains(definition.name.as_str()))
        .map(|definition| {
            (
                definition.name.as_str(),
                definition.span,
                definition.sha256.as_str(),
            )
        })
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    let observed_packages = candidate
        .origins
        .iter()
        .filter_map(|origin| match origin {
            MacroOrigin::Repository { package, .. } => Some(package.clone()),
            _ => None,
        })
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let definition_bound = candidate.definition.as_deref() == Some(path);
    let origin_bound = definition_bound
        && !observed_packages.is_empty()
        && candidate.origins.iter().all(|origin| match origin {
            MacroOrigin::Repository { package, .. } => bound_files
                .iter()
                .any(|file| file.packages.contains(package)),
            _ => false,
        });
    if bound != 1 || !origin_bound {
        Some(MacroBindingFailure::DefinitionMismatch {
            allowance: allowance.name.clone(),
            configured: path.into(),
            observed_definitions: candidate.definition.iter().cloned().collect(),
            observed_packages,
        })
    } else {
        None
    }
}
