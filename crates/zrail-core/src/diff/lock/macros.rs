//! Semantic comparison of content-bound local macro authority.

use std::collections::{BTreeMap, BTreeSet};

use crate::{LockFile, LockedMacroDefinition};

use super::super::{ArchitectureChange, ChangeKind};

pub(super) fn compare(before: &LockFile, after: &LockFile, changes: &mut Vec<ArchitectureChange>) {
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
