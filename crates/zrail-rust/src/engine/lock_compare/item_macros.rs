//! Exact item-macro namespace manifests remain synchronized with reviewed lock state.

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::{Finding, FindingSink, LockFile, LockedItemMacroManifest};

pub(super) fn compare(current: &LockFile, candidate: &LockFile, findings: &mut FindingSink) {
    let old = by_identity(&current.item_macro_manifests);
    let new = by_identity(&candidate.item_macro_manifests);
    for identity in old
        .keys()
        .chain(new.keys())
        .copied()
        .collect::<BTreeSet<_>>()
    {
        match (old.get(&identity), new.get(&identity)) {
            (None, Some(_)) => finding(
                "LOCK-033",
                identity,
                "exact item-macro namespace authority is not reviewed in zrail.lock",
                findings,
            ),
            (Some(_), None) => finding(
                "LOCK-034",
                identity,
                "zrail.lock retains a stale item-macro namespace manifest",
                findings,
            ),
            (Some(left), Some(right)) if left != right => finding(
                "LOCK-035",
                identity,
                "exact item-macro namespace manifest bytes or invocation changed",
                findings,
            ),
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

fn finding(id: &str, identity: (&str, &str), message: &str, findings: &mut FindingSink) {
    findings.push(
        Finding::error(
            id,
            "lock.item-macro-manifest",
            "lock",
            format!("{message}: {} at {}", identity.0, identity.1),
        )
        .with_help("review the exact expansion manifest before updating zrail.lock"),
    );
}
