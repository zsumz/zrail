//! Path-scoped source ownership includes every relevant declared destination.

use zrail_core::{Contract, OwnerKind, glob_matches};

use super::{CallOwnerExplanation, CapabilityOwnerExplanation};

pub(super) fn for_path(contract: &Contract, path: &str) -> Vec<CapabilityOwnerExplanation> {
    let mut owners = contract
        .owners
        .iter()
        .filter(|owner| {
            owner.kind == OwnerKind::Capability
                && owner
                    .within
                    .iter()
                    .any(|pattern| glob_matches(pattern, path))
        })
        .map(|owner| CapabilityOwnerExplanation {
            name: owner.name.clone(),
            capability: owner.selector.clone(),
            allow: owner.allow.clone(),
            allowed_here: owner.allow.iter().any(|allowed| allowed == path),
            reason: owner.reason.clone(),
        })
        .collect::<Vec<_>>();
    owners.sort_by(|left, right| left.name.cmp(&right.name));
    owners
}

pub(super) fn calls_for_path(contract: &Contract, path: &str) -> Vec<CallOwnerExplanation> {
    let mut owners = contract
        .owners
        .iter()
        .filter(|owner| {
            owner.kind == OwnerKind::Call
                && owner
                    .within
                    .iter()
                    .any(|pattern| glob_matches(pattern, path))
        })
        .map(|owner| CallOwnerExplanation {
            name: owner.name.clone(),
            call: owner.selector.clone(),
            allow: owner.allow.clone(),
            allowed_here: owner.allow.iter().any(|allowed| allowed == path),
            reason: owner.reason.clone(),
        })
        .collect::<Vec<_>>();
    owners.sort_by(|left, right| left.name.cmp(&right.name));
    owners
}

pub(super) fn display(owners: &[CapabilityOwnerExplanation]) -> String {
    if owners.is_empty() {
        return "<none>".into();
    }
    owners
        .iter()
        .map(|owner| {
            let access = if owner.allowed_here {
                "allowed here".into()
            } else {
                format!("owned by {}", owner.allow.join(", "))
            };
            format!(
                "{}: {} ({access}; why: {})",
                owner.name, owner.capability, owner.reason
            )
        })
        .collect::<Vec<_>>()
        .join("; ")
}

pub(super) fn display_calls(owners: &[CallOwnerExplanation]) -> String {
    if owners.is_empty() {
        return "<none>".into();
    }
    owners
        .iter()
        .map(|owner| {
            let access = if owner.allowed_here {
                "allowed here".into()
            } else {
                format!("owned by {}", owner.allow.join(", "))
            };
            format!(
                "{}: {} ({access}; why: {})",
                owner.name, owner.call, owner.reason
            )
        })
        .collect::<Vec<_>>()
        .join("; ")
}
