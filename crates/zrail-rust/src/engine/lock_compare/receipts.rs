//! Exact lock drift for execution-backed test mirrors.

use std::collections::BTreeMap;

use zrail_core::{Finding, FindingSink, LockFile, LockedExecutionReceipt};

pub(super) fn compare(current: &LockFile, candidate: &LockFile, findings: &mut FindingSink) {
    let old = by_production(&current.execution_receipts);
    let new = by_production(&candidate.execution_receipts);
    for production in new.keys().filter(|path| !old.contains_key(*path)) {
        findings.push(Finding::error(
            "LOCK-040",
            "lock.execution-receipt",
            "lock",
            format!("test mirror for {production:?} has no reviewed execution receipt"),
        ));
    }
    for production in old.keys().filter(|path| !new.contains_key(*path)) {
        findings.push(Finding::error(
            "LOCK-041",
            "lock.execution-receipt",
            "lock",
            format!("zrail.lock retains a stale execution receipt for {production:?}"),
        ));
    }
    for (production, before) in old {
        if new.get(production).is_some_and(|after| before != *after) {
            findings.push(Finding::error(
                "LOCK-042",
                "lock.execution-receipt",
                "lock",
                format!("reviewed execution receipt for {production:?} changed"),
            ));
        }
    }
}

fn by_production(receipts: &[LockedExecutionReceipt]) -> BTreeMap<&str, &LockedExecutionReceipt> {
    receipts
        .iter()
        .map(|receipt| (receipt.production.as_str(), receipt))
        .collect()
}
