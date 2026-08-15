//! Exact lock drift for complete generated-provenance manifests.

use std::collections::BTreeMap;

use zrail_core::{Finding, FindingSink, LockFile, LockedGeneratedSource};

pub(super) fn compare(current: &LockFile, candidate: &LockFile, findings: &mut FindingSink) {
    let old = by_root(&current.generated);
    let new = by_root(&candidate.generated);
    for root in new.keys().filter(|root| !old.contains_key(*root)) {
        findings.push(Finding::error(
            "LOCK-011",
            "lock.generated-provenance",
            "lock",
            format!("generated root {root:?} has no reviewed provenance manifest in zrail.lock"),
        ));
    }
    for root in old.keys().filter(|root| !new.contains_key(*root)) {
        findings.push(Finding::error(
            "LOCK-012",
            "lock.generated-provenance",
            "lock",
            format!("zrail.lock retains stale generated provenance for {root:?}"),
        ));
    }
    for (root, old_digest) in old {
        let Some(new_digest) = new.get(root) else {
            continue;
        };
        if old_digest != *new_digest {
            findings.push(Finding::error(
                "LOCK-013",
                "lock.generated-provenance",
                "lock",
                format!("generated provenance manifest changed for {root:?}"),
            ));
        }
    }
}

fn by_root(generated: &[LockedGeneratedSource]) -> BTreeMap<&str, &str> {
    generated
        .iter()
        .map(|generated| (generated.root.as_str(), generated.manifest_sha256.as_str()))
        .collect()
}
