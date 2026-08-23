//! Optional `macro_rules!` definition hints must narrow the observed candidate.

use zrail_core::MacroExpansionAllow;

use crate::source::{MacroCandidate, MacroOrigin};

use super::{super::RuleContext, failure::MacroBindingFailure};

pub(super) fn failure(
    context: &RuleContext<'_>,
    candidate: &MacroCandidate,
    allowance: &MacroExpansionAllow,
) -> Option<MacroBindingFailure> {
    let path = allowance.definition.as_deref()?;
    let bound_file = context
        .source
        .files
        .iter()
        .find(|file| file.relative == path);
    let definition_names = candidate
        .policy_names()
        .map(|name| name.rsplit("::").next().unwrap_or(name))
        .collect::<std::collections::BTreeSet<_>>();
    let bound = bound_file.map_or(0, |file| {
        file.macro_definitions
            .iter()
            .filter(|definition| definition_names.contains(definition.name.as_str()))
            .count()
    });
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
    let definition_bound = candidate
        .definition
        .as_deref()
        .is_none_or(|observed| observed == path);
    let origin_bound = definition_bound
        && !observed_packages.is_empty()
        && candidate.origins.iter().all(|origin| match origin {
            MacroOrigin::Repository { package, .. } => {
                bound_file.is_some_and(|file| file.packages.contains(package))
            }
            _ => false,
        });
    if bound != 1 || !origin_bound {
        Some(MacroBindingFailure::DefinitionMismatch {
            allowance: allowance.name.clone(),
            configured: path.into(),
            observed_packages,
        })
    } else {
        None
    }
}
