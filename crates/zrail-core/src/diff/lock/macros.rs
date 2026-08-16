//! Semantic comparison of package-bound and legacy macro authority.

use std::collections::{BTreeMap, BTreeSet};

use crate::{LockFile, LockedMacroDefinition, LockedMacroImplementation};

use super::super::{ArchitectureChange, ChangeKind};

pub(super) fn compare(before: &LockFile, after: &LockFile, changes: &mut Vec<ArchitectureChange>) {
    compare_implementations(before, after, changes);
    let old = by_identity(&before.macros);
    let new = by_identity(&after.macros);
    for identity in old
        .keys()
        .chain(new.keys())
        .cloned()
        .collect::<BTreeSet<_>>()
    {
        match (old.get(&identity), new.get(&identity)) {
            (None, Some(_)) => changes.push(ArchitectureChange::new(
                ChangeKind::Grant,
                "rust.macro-definition",
                &identity,
                "local macro implementation became trusted",
            )),
            (Some(_), None) => changes.push(ArchitectureChange::new(
                ChangeKind::Revoke,
                "rust.macro-definition",
                &identity,
                "local macro implementation is no longer trusted",
            )),
            (Some(left), Some(right)) if left.sha256 != right.sha256 => changes.push(
                ArchitectureChange::new(
                    ChangeKind::Unknown,
                    "rust.macro-definition",
                    &identity,
                    "trusted local macro implementation changed",
                )
                .values(&left.sha256, &right.sha256),
            ),
            _ => {}
        }
    }
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

fn by_identity(definitions: &[LockedMacroDefinition]) -> BTreeMap<String, &LockedMacroDefinition> {
    definitions
        .iter()
        .map(|definition| {
            (
                format!(
                    "{}:{}:{}",
                    definition.path, definition.name, definition.ordinal
                ),
                definition,
            )
        })
        .collect()
}
