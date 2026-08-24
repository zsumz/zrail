//! Exact item-macro namespace authority changes remain visible.

use std::collections::{BTreeMap, BTreeSet};

use crate::{LockFile, LockedItemMacroManifest};

use super::super::{ArchitectureChange, ChangeKind};

pub(super) fn compare(before: &LockFile, after: &LockFile, changes: &mut Vec<ArchitectureChange>) {
    let old = by_identity(&before.item_macro_manifests);
    let new = by_identity(&after.item_macro_manifests);
    for identity in old
        .keys()
        .chain(new.keys())
        .copied()
        .collect::<BTreeSet<_>>()
    {
        let subject = format!("{}:{}", identity.0, identity.1);
        match (old.get(&identity), new.get(&identity)) {
            (None, Some(_)) => changes.push(ArchitectureChange::new(
                ChangeKind::Revoke,
                "rust.item-macro-manifest",
                subject,
                "item-macro namespace became exactly manifested",
            )),
            (Some(_), None) => changes.push(ArchitectureChange::new(
                ChangeKind::Grant,
                "rust.item-macro-manifest",
                subject,
                "exact item-macro namespace authority was removed",
            )),
            (Some(left), Some(right)) if left != right => changes.push(ArchitectureChange::new(
                ChangeKind::Unknown,
                "rust.item-macro-manifest",
                subject,
                "item-macro manifest, definition, invocation, guard, or compilation authority changed",
            )),
            _ => {}
        }
    }
}

fn by_identity(
    values: &[LockedItemMacroManifest],
) -> BTreeMap<(&str, &str), &LockedItemMacroManifest> {
    values
        .iter()
        .map(|value| ((value.name.as_str(), value.invocation_path.as_str()), value))
        .collect()
}
