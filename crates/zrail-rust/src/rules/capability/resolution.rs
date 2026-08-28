//! Generic associated calls fail closed only when configured call authority relies on them.

use zrail_core::{FindingSink, OwnerKind, PolicyReachability, glob_matches};

use crate::source::{
    AssociatedOccurrenceKind, CallResolutionFact, CallResolutionKind, ProviderAuthority,
    RustFileFacts,
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
                        boundary.associated_candidates.iter().any(|candidate| {
                            candidate_relevant(selector, candidate, boundary.occurrence)
                        })
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
            .any(|candidate| candidate_relevant(&owner.selector, candidate, boundary.occurrence))
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

fn candidate_relevant(
    selector: &str,
    candidate: &crate::source::GenericAssociatedCandidate,
    occurrence: Option<AssociatedOccurrenceKind>,
) -> bool {
    candidate_matches(selector, candidate)
        || (!candidate.provider_complete
            && incomplete_provider_matches(selector, candidate, occurrence))
}

fn incomplete_provider_matches(
    selector: &str,
    candidate: &crate::source::GenericAssociatedCandidate,
    occurrence: Option<AssociatedOccurrenceKind>,
) -> bool {
    let selectors = selector_authorities(selector);
    let same_authority = candidate
        .provider_authorities
        .iter()
        .filter(|authority| **authority != ProviderAuthority::Unknown)
        .any(|authority| selectors.contains(authority));
    (occurrence != Some(AssociatedOccurrenceKind::TypeReference) && same_authority)
        || (candidate
            .provider_authorities
            .contains(&ProviderAuthority::Unknown)
            && same_associated_item(selector, &candidate.name))
}

fn same_associated_item(selector: &str, candidate: &str) -> bool {
    last_segment(selector) == last_segment(candidate)
}

fn last_segment(path: &str) -> Option<&str> {
    path.rsplit("::")
        .next()
        .map(|segment| segment.strip_prefix("r#").unwrap_or(segment))
}

fn selector_authorities(selector: &str) -> std::collections::BTreeSet<ProviderAuthority> {
    let absolute = selector.starts_with("::");
    let root = selector
        .trim_start_matches("::")
        .split("::")
        .next()
        .map(|root| root.strip_prefix("r#").unwrap_or(root));
    match root {
        Some("crate" | "self" | "super") => [ProviderAuthority::LocalCrate].into(),
        Some(root @ ("std" | "core" | "alloc")) => {
            [ProviderAuthority::ExternalRoot(root.into())].into()
        }
        Some(root) if !root.is_empty() && absolute => {
            [ProviderAuthority::ExternalRoot(root.into())].into()
        }
        Some(root) if !root.is_empty() => [
            ProviderAuthority::LocalCrate,
            ProviderAuthority::ExternalRoot(root.into()),
        ]
        .into(),
        _ => [ProviderAuthority::Unknown].into(),
    }
}

#[cfg(test)]
#[path = "resolution_test.rs"]
mod resolution_test;
