//! Generic associated calls fail closed only when configured call authority relies on them.

use zrail_core::{FindingSink, OwnerKind, PolicyReachability, glob_matches};

use crate::source::{CallResolutionFact, CallResolutionKind, RustFileFacts};

use super::super::RuleContext;

pub(super) fn check(context: &RuleContext<'_>, findings: &mut FindingSink) {
    for file in &context.source.files {
        for boundary in file
            .call_resolutions
            .iter()
            .filter(|boundary| boundary.kind == CallResolutionKind::GenericAssociatedItem)
        {
            if context
                .contract
                .owners
                .iter()
                .any(|owner| owner_relies_on(owner, file, boundary))
            {
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
    owner.kind == OwnerKind::Call
        && owner
            .within
            .iter()
            .any(|pattern| glob_matches(pattern, &file.relative))
        && (owner.reachability == PolicyReachability::All
            || (file.reachability.is_production() && boundary.guard.is_production_applicable()))
        && same_item(&owner.selector, &boundary.written)
}

fn same_item(selector: &str, written: &str) -> bool {
    let selector = super::normalized_path(selector);
    let written = super::normalized_path(written);
    selector.rsplit("::").next() == written.rsplit("::").next()
}

#[cfg(test)]
#[path = "resolution_test.rs"]
mod resolution_test;
