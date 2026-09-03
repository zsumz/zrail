//! Semantic comparison of package-bound macro authority.

use std::collections::{BTreeMap, BTreeSet};

use crate::{Contract, CrateRootSource, LockFile, LockedMacroImplementation, LockedMacroSource};

use super::super::{ArchitectureChange, ChangeKind};

pub(super) fn compare(
    before_contract: &Contract,
    before: &LockFile,
    after_contract: &Contract,
    after: &LockFile,
    changes: &mut Vec<ArchitectureChange>,
) {
    let expansion_authority = enforced(before_contract) && enforced(after_contract);
    if expansion_authority
        || repository_item_macros(before_contract)
        || repository_item_macros(after_contract)
    {
        compare_implementations(before, after, changes);
    }
    if expansion_authority {
        compare_sources(before, after, changes);
    }
}

fn enforced(contract: &Contract) -> bool {
    contract.source.rust.macros.mode == crate::MacroExpansionMode::DenyUnreviewed
}

fn repository_item_macros(contract: &Contract) -> bool {
    contract
        .source
        .rust
        .item_macros
        .iter()
        .any(|allowance| matches!(allowance.source, Some(CrateRootSource::Repository { .. })))
}

fn compare_sources(before: &LockFile, after: &LockFile, changes: &mut Vec<ArchitectureChange>) {
    let old = sources(&before.macro_sources);
    let new = sources(&after.macro_sources);
    for allowance in old
        .keys()
        .chain(new.keys())
        .copied()
        .collect::<BTreeSet<_>>()
    {
        compare_source_group(
            allowance,
            old.get(allowance).map(Vec::as_slice).unwrap_or_default(),
            new.get(allowance).map(Vec::as_slice).unwrap_or_default(),
            changes,
        );
    }
}

fn compare_source_group(
    allowance: &str,
    old: &[&LockedMacroSource],
    new: &[&LockedMacroSource],
    changes: &mut Vec<ArchitectureChange>,
) {
    if let ([left], [right]) = (old, new) {
        compare_source(allowance, left, right, changes);
        return;
    }
    let old = source_identities(old);
    let new = source_identities(new);
    for identity in old.keys().chain(new.keys()).collect::<BTreeSet<_>>() {
        let subject = format!("{allowance} [{identity}]");
        match (old.get(identity), new.get(identity)) {
            (None, Some(_)) => changes.push(ArchitectureChange::new(
                ChangeKind::Revoke,
                "rust.macro-source",
                &subject,
                "macro authority became bound to an exact Cargo.lock package",
            )),
            (Some(_), None) => changes.push(ArchitectureChange::new(
                ChangeKind::Grant,
                "rust.macro-source",
                &subject,
                "exact Cargo.lock macro authority was removed",
            )),
            (Some(left), Some(right)) => compare_source(&subject, left, right, changes),
            _ => {}
        }
    }
}

fn compare_source(
    subject: &str,
    left: &LockedMacroSource,
    right: &LockedMacroSource,
    changes: &mut Vec<ArchitectureChange>,
) {
    if left != right {
        changes.push(
            ArchitectureChange::new(
                ChangeKind::Unknown,
                "rust.macro-source",
                subject,
                "the exact macro implementation package changed",
            )
            .values(package_label(left), package_label(right)),
        );
    }
}

fn sources(values: &[LockedMacroSource]) -> BTreeMap<&str, Vec<&LockedMacroSource>> {
    let mut sources = BTreeMap::<_, Vec<_>>::new();
    for value in values {
        sources
            .entry(value.allowance.as_str())
            .or_default()
            .push(value);
    }
    sources
}

fn source_identities<'a>(
    values: &[&'a LockedMacroSource],
) -> BTreeMap<String, &'a LockedMacroSource> {
    values
        .iter()
        .map(|value| (source_identity(value), *value))
        .collect()
}

fn source_identity(value: &LockedMacroSource) -> String {
    format!("{} {} ({})", value.package, value.version, value.source)
}

fn package_label(value: &LockedMacroSource) -> String {
    let checksum = value.checksum.as_deref().unwrap_or("no checksum");
    format!(
        "{} {} ({}, {checksum})",
        value.package, value.version, value.source
    )
}

fn compare_implementations(
    before: &LockFile,
    after: &LockFile,
    changes: &mut Vec<ArchitectureChange>,
) {
    let old = implementations(&before.macro_implementations);
    let new = implementations(&after.macro_implementations);
    for identity in old
        .keys()
        .chain(new.keys())
        .cloned()
        .collect::<BTreeSet<_>>()
    {
        match (old.get(&identity), new.get(&identity)) {
            (None, Some(_)) => changes.push(ArchitectureChange::new(
                ChangeKind::Grant,
                "rust.macro-implementation",
                &identity,
                "repository macro implementation package became trusted",
            )),
            (Some(_), None) => changes.push(ArchitectureChange::new(
                ChangeKind::Revoke,
                "rust.macro-implementation",
                &identity,
                "repository macro implementation package is no longer trusted",
            )),
            (Some(left), Some(right)) if left.inputs_sha256 != right.inputs_sha256 => {
                changes.push(
                    ArchitectureChange::new(
                        ChangeKind::Unknown,
                        "rust.macro-implementation",
                        &identity,
                        "trusted repository macro implementation package changed",
                    )
                    .values(&left.inputs_sha256, &right.inputs_sha256),
                );
            }
            _ => {}
        }
    }
}

fn implementations(
    values: &[LockedMacroImplementation],
) -> BTreeMap<String, &LockedMacroImplementation> {
    values
        .iter()
        .map(|value| (format!("{}:{}", value.directory, value.package), value))
        .collect()
}
