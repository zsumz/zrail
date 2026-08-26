//! Exact capabilities and direct calls remain confined to declared source owners.

mod scope;

use std::collections::BTreeSet;

use zrail_core::{Finding, FindingSink, OwnerKind, PolicyReachability, glob_matches};

use super::{RuleContext, path_matches};

pub(super) fn check(context: &RuleContext<'_>, findings: &mut FindingSink) {
    for owner in context
        .contract
        .owners
        .iter()
        .filter(|owner| owner.kind != OwnerKind::Directory)
    {
        check_owner(context, owner, findings);
    }
}

fn check_owner(
    context: &RuleContext<'_>,
    owner: &zrail_core::OwnerContract,
    findings: &mut FindingSink,
) {
    let allowed = owner
        .allow
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let files = context
        .source
        .files
        .iter()
        .filter(|file| {
            owner
                .within
                .iter()
                .any(|pattern| glob_matches(pattern, &file.relative))
        })
        .collect::<Vec<_>>();
    scope::reject_stale_scope(owner, &files, findings);
    scope::reject_missing_owners(owner, &allowed, &files, findings);
    let mut used_allowed = BTreeSet::new();
    for file in files {
        if allowed.contains(file.relative.as_str()) {
            let used = match owner.kind {
                OwnerKind::Call => super::ownership_call::check(owner, file, findings),
                OwnerKind::Capability => !matching_capability(owner, file).is_empty(),
                OwnerKind::Directory => false,
                OwnerKind::TypeConstruction
                | OwnerKind::MethodName
                | OwnerKind::FieldRead
                | OwnerKind::FieldWrite
                | OwnerKind::FieldMutableBorrow
                | OwnerKind::FieldMutation
                | OwnerKind::FieldAuthority => {
                    super::ownership_operation::check(owner, file, findings)
                }
            };
            if used {
                used_allowed.insert(file.relative.as_str());
            }
        } else {
            for path in owned_evidence(owner, file) {
                findings.push(
                    Finding::error(
                        "OWN-003",
                        &owner.name,
                        "ownership",
                        scope::violation(owner, &path.name),
                    )
                    .at(&file.relative, path.span)
                    .because(&owner.reason)
                    .with_analysis(path.quality)
                    .with_help(scope::owner_help(owner)),
                );
            }
        }
    }
    scope::reject_unused_owners(owner, &allowed, &used_allowed, context, findings);
}

fn owned_evidence<'a>(
    owner: &zrail_core::OwnerContract,
    file: &'a crate::source::RustFileFacts,
) -> Vec<&'a crate::source::ObservedFact> {
    match owner.kind {
        OwnerKind::Capability => return matching_capability(owner, file),
        OwnerKind::TypeConstruction
        | OwnerKind::MethodName
        | OwnerKind::FieldRead
        | OwnerKind::FieldWrite
        | OwnerKind::FieldMutableBorrow
        | OwnerKind::FieldMutation
        | OwnerKind::FieldAuthority => {
            return super::ownership_operation::matching(owner, file);
        }
        OwnerKind::Call | OwnerKind::Directory => {}
    }
    let calls = matching(owner, file, &file.calls);
    if calls.is_empty() {
        matching(owner, file, &file.paths)
    } else {
        calls
    }
}

fn matching<'a>(
    owner: &zrail_core::OwnerContract,
    file: &crate::source::RustFileFacts,
    facts: &'a [crate::source::ObservedFact],
) -> Vec<&'a crate::source::ObservedFact> {
    facts
        .iter()
        .filter(|fact| fact_applies(owner, file, fact) && path_matches(&owner.selector, fact))
        .collect()
}

pub(crate) fn matching_capability<'a>(
    owner: &zrail_core::OwnerContract,
    file: &'a crate::source::RustFileFacts,
) -> Vec<&'a crate::source::ObservedFact> {
    matching(owner, file, &file.paths)
}

pub(super) fn fact_applies(
    owner: &zrail_core::OwnerContract,
    file: &crate::source::RustFileFacts,
    fact: &crate::source::ObservedFact,
) -> bool {
    owner.reachability == PolicyReachability::All
        || fact.is_production_applicable(file.reachability)
}
