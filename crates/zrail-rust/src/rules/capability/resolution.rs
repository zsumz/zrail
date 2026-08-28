//! Generic associated calls fail closed only when configured call authority relies on them.

use zrail_core::{FindingSink, OwnerKind, PolicyReachability, glob_matches};

use crate::source::{
    AssociatedOccurrenceKind, CallResolutionFact, CallResolutionKind, RustFileFacts,
};

use super::super::RuleContext;

pub(super) fn check(context: &RuleContext<'_>, findings: &mut FindingSink) {
    for file in &context.source.files {
        for boundary in file
            .call_resolutions
            .iter()
            .filter(|boundary| boundary.kind == CallResolutionKind::GenericAssociatedItem)
        {
            let owner_relies = context
                .contract
                .owners
                .iter()
                .any(|owner| owner_relies_on(owner, file, boundary));
            let scope_relies = context.contract.scopes.iter().any(|scope| {
                super::matches_scope(&file.relative, &scope.include, &scope.exclude)
                    && scope.symbols.deny.iter().any(|selector| {
                        boundary
                            .associated_candidates
                            .iter()
                            .any(|candidate| candidate_matches(selector, candidate))
                    })
            });
            if owner_relies || scope_relies {
                findings.push(crate::source::call_resolution_finding(
                    &file.relative,
                    boundary,
                ));
            }
        }
    }
}

fn owner_relies_on(
    owner: &zrail_core::OwnerContract,
    file: &RustFileFacts,
    boundary: &CallResolutionFact,
) -> bool {
    owner_kind_applies(owner.kind, boundary.occurrence)
        && owner
            .within
            .iter()
            .any(|pattern| glob_matches(pattern, &file.relative))
        && (owner.reachability == PolicyReachability::All
            || (file.reachability.is_production() && boundary.guard.is_production_applicable()))
        && boundary
            .associated_candidates
            .iter()
            .any(|candidate| candidate_matches(&owner.selector, candidate))
}

fn owner_kind_applies(kind: OwnerKind, occurrence: Option<AssociatedOccurrenceKind>) -> bool {
    matches!(
        (kind, occurrence),
        (OwnerKind::Call, Some(AssociatedOccurrenceKind::DirectCall))
            | (
                OwnerKind::Capability,
                Some(AssociatedOccurrenceKind::ValueReference)
            )
    )
}

fn candidate_matches(
    selector: &str,
    candidate: &crate::source::GenericAssociatedCandidate,
) -> bool {
    let selector = super::normalized_path(selector);
    candidate.policy_names().any(|name| {
        let name = super::normalized_path(name);
        name == selector || name.starts_with(&format!("{selector}::"))
    })
}

#[cfg(test)]
#[path = "resolution_test.rs"]
mod resolution_test;
