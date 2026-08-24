//! Owner scopes and stale declarations produce shared policy diagnostics.

use std::collections::BTreeSet;

use zrail_core::{Finding, FindingSink, OwnerKind, PolicyReachability, glob_matches};

use super::RuleContext;

pub(super) fn violation(owner: &zrail_core::OwnerContract, observed: &str) -> String {
    let resource = match owner.kind {
        OwnerKind::Call => "call authority",
        OwnerKind::Capability => "capability",
        OwnerKind::TypeConstruction => "type construction authority",
        OwnerKind::MethodName => "written method-name authority",
        OwnerKind::FieldRead => "field-read authority",
        OwnerKind::FieldWrite => "field-write authority",
        OwnerKind::FieldMutableBorrow => "field mutable-borrow authority",
        OwnerKind::FieldAuthority => "field authority",
        OwnerKind::Directory => "directory authority",
    };
    format!(
        "source reaches owned {resource} {observed}; allowed owner: {}",
        owner.allow.join(", ")
    )
}

pub(super) fn owner_help(owner: &zrail_core::OwnerContract) -> &'static str {
    match owner.kind {
        OwnerKind::Call => "move the call into a declared owner and expose a narrow operation",
        OwnerKind::Capability => {
            "move the capability use into a declared owner and pass facts inward"
        }
        OwnerKind::TypeConstruction => {
            "move construction into a declared owner and expose a narrow constructor"
        }
        OwnerKind::MethodName => "move calls with this written method name into a declared owner",
        OwnerKind::FieldRead => "move field reads into a declared owner and expose a narrow query",
        OwnerKind::FieldWrite | OwnerKind::FieldMutableBorrow => {
            "move field mutation into a declared owner and expose a narrow operation"
        }
        OwnerKind::FieldAuthority => {
            "move field access into a declared owner and expose narrow operations"
        }
        OwnerKind::Directory => "move the resource into its declared directory owner",
    }
}

pub(super) fn reject_unused_owners(
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
        match owner.kind {
            OwnerKind::Call => "direct invocation",
            OwnerKind::Capability => "use",
            OwnerKind::TypeConstruction => "construction",
            OwnerKind::MethodName => "call with the written method name",
            OwnerKind::FieldRead => "field read",
            OwnerKind::FieldWrite => "field write",
            OwnerKind::FieldMutableBorrow => "field mutable borrow",
            OwnerKind::FieldAuthority => "field access",
            OwnerKind::Directory => "directory use",
        },
        owner.selector,
    )
}

pub(super) fn reject_stale_scope(
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

pub(super) fn reject_missing_owners(
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
