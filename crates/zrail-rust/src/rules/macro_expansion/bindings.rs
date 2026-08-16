//! Optional `macro_rules!` definition hints must narrow exact repository origins.

use std::collections::BTreeMap;

use zrail_core::{Finding, FindingSink, MacroExpansionAllow};

use crate::source::{MacroOrigin, Reachability};

use super::super::RuleContext;

pub(super) fn validate(
    context: &RuleContext<'_>,
    allowed: &BTreeMap<&str, &MacroExpansionAllow>,
    findings: &mut FindingSink,
) {
    for allowance in allowed
        .values()
        .filter(|allowance| allowance.definition.is_some())
    {
        validate_allowance(context, allowance, findings);
    }
}

fn validate_allowance(
    context: &RuleContext<'_>,
    allowance: &MacroExpansionAllow,
    findings: &mut FindingSink,
) {
    let path = allowance.definition.as_deref().unwrap_or_default();
    let leaf = allowance
        .name
        .rsplit("::")
        .next()
        .unwrap_or(&allowance.name);
    let bound_file = context
        .source
        .files
        .iter()
        .find(|file| file.relative == path);
    let bound = bound_file.map_or(0, |file| {
        file.macro_definitions
            .iter()
            .filter(|definition| definition.name == leaf)
            .count()
    });
    let origins = context
        .source
        .files
        .iter()
        .filter(|file| file.reachability != Reachability::Unreachable)
        .flat_map(|file| &file.macro_expansions)
        .filter(|expansion| expansion.policy_names().any(|name| name == allowance.name))
        .flat_map(|expansion| &expansion.origins)
        .collect::<Vec<_>>();
    let origin_bound = !origins.is_empty()
        && origins.iter().all(|origin| match origin {
            MacroOrigin::Repository { package, .. } => {
                bound_file.is_some_and(|file| file.packages.contains(package))
            }
            _ => false,
        });
    if bound != 1 || !origin_bound {
        findings.push(
            Finding::error(
                "RUST-MACRO-005",
                "rust.macro-definition",
                "source",
                format!(
                    "repository macro allowance {:?} resolves to {bound} matching definitions in {path:?}, but that path is not its exact observed implementation package",
                    allowance.name,
                ),
            )
            .because(&allowance.reason)
            .with_help("bind the optional definition hint to one macro_rules! definition in the observed repository implementation package"),
        );
    }
}
