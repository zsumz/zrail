//! Semantic comparison of package-bound macro authority.

use std::collections::{BTreeMap, BTreeSet};

use crate::{LockFile, LockedMacroImplementation, LockedMacroSource};

use super::super::{ArchitectureChange, ChangeKind};

pub(super) fn compare(before: &LockFile, after: &LockFile, changes: &mut Vec<ArchitectureChange>) {
    compare_implementations(before, after, changes);
    compare_sources(before, after, changes);
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
        match (old.get(allowance), new.get(allowance)) {
            (None, Some(_)) => changes.push(ArchitectureChange::new(
                ChangeKind::Revoke,
                "rust.macro-source",
                allowance,
                "macro authority became bound to an exact Cargo.lock package",
            )),
            (Some(_), None) => changes.push(ArchitectureChange::new(
                ChangeKind::Grant,
                "rust.macro-source",
                allowance,
                "exact Cargo.lock macro authority was removed",
            )),
            (Some(left), Some(right)) if left != right => changes.push(
                ArchitectureChange::new(
                    ChangeKind::Unknown,
                    "rust.macro-source",
                    allowance,
                    "the exact macro implementation package changed",
                )
                .values(package_label(left), package_label(right)),
            ),
            _ => {}
        }
    }
}

fn sources(values: &[LockedMacroSource]) -> BTreeMap<&str, &LockedMacroSource> {
    values
        .iter()
        .map(|value| (value.allowance.as_str(), value))
        .collect()
}

fn package_label(value: &LockedMacroSource) -> String {
    format!("{} {} ({})", value.package, value.version, value.source)
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
            (Some(left), Some(right)) if left.manifest_sha256 != right.manifest_sha256 => {
                changes.push(
                    ArchitectureChange::new(
                        ChangeKind::Unknown,
                        "rust.macro-implementation",
                        &identity,
                        "trusted repository macro implementation package changed",
                    )
                    .values(&left.manifest_sha256, &right.manifest_sha256),
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
