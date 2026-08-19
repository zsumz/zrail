//! Exact lock drift for package-bound macro implementation manifests.

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::{Finding, FindingSink, LockFile, LockedMacroImplementation};

pub(super) fn compare(current: &LockFile, candidate: &LockFile, findings: &mut FindingSink) {
    compare_implementations(current, candidate, findings);
}

fn compare_implementations(current: &LockFile, candidate: &LockFile, findings: &mut FindingSink) {
    let old = implementations(&current.macro_implementations);
    let new = implementations(&candidate.macro_implementations);
    for identity in old
        .keys()
        .chain(new.keys())
        .cloned()
        .collect::<BTreeSet<_>>()
    {
        match (old.get(&identity), new.get(&identity)) {
            (None, Some(_)) => findings.push(Finding::error(
                "LOCK-021",
                "lock.macro-implementation",
                "lock",
                format!(
                    "repository macro implementation {identity:?} is not reviewed in zrail.lock"
                ),
            )),
            (Some(_), None) => findings.push(Finding::error(
                "LOCK-022",
                "lock.macro-implementation",
                "lock",
                format!("zrail.lock retains stale repository macro implementation {identity:?}"),
            )),
            (Some(left), Some(right)) if left.manifest_sha256 != right.manifest_sha256 => {
                findings.push(Finding::error(
                    "LOCK-023",
                    "lock.macro-implementation",
                    "lock",
                    format!("reviewed repository macro implementation {identity:?} changed"),
                ));
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
