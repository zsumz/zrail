//! Exact capabilities and direct calls remain confined to declared source owners.

use std::collections::BTreeSet;

use zrail_core::{Finding, FindingSink, OwnerKind, PolicyReachability, glob_matches};

use super::{RuleContext, path_matches};

pub(super) fn check(context: &RuleContext<'_>, findings: &mut FindingSink) {
    for owner in context
        .contract
        .owners
        .iter()
        .filter(|owner| matches!(owner.kind, OwnerKind::Call | OwnerKind::Capability))
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
    reject_stale_scope(owner, &files, findings);
    reject_missing_owners(owner, &allowed, &files, findings);
    let mut used_allowed = BTreeSet::new();
    for file in files {
        if allowed.contains(file.relative.as_str()) {
            let used = match owner.kind {
                OwnerKind::Call => super::ownership_call::check(owner, file, findings),
                OwnerKind::Capability => !matching(owner, file, &file.paths).is_empty(),
                OwnerKind::Directory => false,
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
                        violation(owner, &path.name),
                    )
                    .at(&file.relative, path.span)
                    .because(&owner.reason)
                    .with_analysis(path.quality)
                    .with_help(owner_help(owner)),
                );
            }
        }
    }
    reject_unused_owners(owner, &allowed, &used_allowed, context, findings);
}

fn owned_evidence<'a>(
    owner: &zrail_core::OwnerContract,
    file: &'a crate::source::RustFileFacts,
) -> Vec<&'a crate::source::ObservedFact> {
    if owner.kind == OwnerKind::Capability {
        return matching(owner, file, &file.paths);
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

pub(super) fn fact_applies(
    owner: &zrail_core::OwnerContract,
    file: &crate::source::RustFileFacts,
    fact: &crate::source::ObservedFact,
) -> bool {
    owner.reachability == PolicyReachability::All
        || fact.is_production_applicable(file.reachability)
}

fn violation(owner: &zrail_core::OwnerContract, observed: &str) -> String {
    let resource = if owner.kind == OwnerKind::Call {
        "call authority"
    } else {
        "capability"
    };
    format!(
        "source reaches owned {resource} {observed}; allowed owner: {}",
        owner.allow.join(", ")
    )
}

fn owner_help(owner: &zrail_core::OwnerContract) -> &'static str {
    if owner.kind == OwnerKind::Call {
        "move the call into a declared owner and expose a narrow operation"
    } else {
        "move the capability use into a declared owner and pass facts inward"
    }
}

fn reject_unused_owners(
    owner: &zrail_core::OwnerContract,
    allowed: &BTreeSet<&str>,
    used: &BTreeSet<&str>,
    context: &RuleContext<'_>,
    findings: &mut FindingSink,
) {
    for path in allowed.difference(used).filter(|path| {
        context
            .source
            .files
            .iter()
            .any(|file| file.relative.as_str() == **path)
    }) {
        findings.push(
            Finding::error(
                "OWN-004",
                &owner.name,
                "ownership",
                unused_owner_message(owner, path),
            )
            .at(*path, None)
            .because(&owner.reason),
        );
    }
}

fn unused_owner_message(owner: &zrail_core::OwnerContract, path: &str) -> String {
    if owner.reachability == PolicyReachability::Production {
        return format!(
            "allowed owner {path:?} has no production-reachable use of {}",
            owner.selector
        );
    }
    format!(
        "allowed owner {path:?} reaches no {} of {}",
        if owner.kind == OwnerKind::Call {
            "direct invocation"
        } else {
            "use"
        },
        owner.selector,
    )
}

fn reject_stale_scope(
    owner: &zrail_core::OwnerContract,
    files: &[&crate::source::RustFileFacts],
    findings: &mut FindingSink,
) {
    for pattern in &owner.within {
        if files
            .iter()
            .any(|file| glob_matches(pattern, &file.relative))
        {
            continue;
        }
        findings.push(
            Finding::error(
                "OWN-004",
                &owner.name,
                "ownership",
                format!("owner scope {pattern:?} matches no Rust source"),
            )
            .because(&owner.reason),
        );
    }
}

fn reject_missing_owners(
    owner: &zrail_core::OwnerContract,
    allowed: &BTreeSet<&str>,
    files: &[&crate::source::RustFileFacts],
    findings: &mut FindingSink,
) {
    let existing = files
        .iter()
        .map(|file| file.relative.as_str())
        .collect::<BTreeSet<_>>();
    for path in allowed.difference(&existing) {
        findings.push(
            Finding::error(
                "OWN-002",
                &owner.name,
                "ownership",
                format!("owner policy names missing Rust source {path:?}"),
            )
            .at(*path, None)
            .because(&owner.reason),
        );
    }
}
