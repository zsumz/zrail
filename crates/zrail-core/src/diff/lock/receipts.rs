//! Semantic comparison of content-bound execution receipts.

use std::collections::{BTreeMap, BTreeSet};

use crate::{LockFile, LockedExecutionReceipt};

use super::super::{ArchitectureChange, ChangeKind};

pub(super) fn compare(before: &LockFile, after: &LockFile, changes: &mut Vec<ArchitectureChange>) {
    let old = by_production(&before.execution_receipts);
    let new = by_production(&after.execution_receipts);
    for production in old
        .keys()
        .chain(new.keys())
        .copied()
        .collect::<BTreeSet<_>>()
    {
        match (old.get(production), new.get(production)) {
            (None, Some(_)) => changes.push(ArchitectureChange::new(
                ChangeKind::Revoke,
                "rust.test-mirror-receipt-lock",
                production,
                "test mirror execution receipt became content-addressed",
            )),
            (Some(_), None) => changes.push(ArchitectureChange::new(
                ChangeKind::Grant,
                "rust.test-mirror-receipt-lock",
                production,
                "test mirror execution receipt is no longer content-addressed",
            )),
            (Some(left), Some(right)) if left != right => changes.push(
                ArchitectureChange::new(
                    ChangeKind::Unknown,
                    "rust.test-mirror-receipt-lock",
                    production,
                    "reviewed execution receipt contents or identity changed",
                )
                .values(value(left), value(right)),
            ),
            _ => {}
        }
    }
}

fn by_production(receipts: &[LockedExecutionReceipt]) -> BTreeMap<&str, &LockedExecutionReceipt> {
    receipts
        .iter()
        .map(|receipt| (receipt.production.as_str(), receipt))
        .collect()
}

fn value(receipt: &LockedExecutionReceipt) -> String {
    format!(
        "{}::{}@{}#{}",
        receipt.test, receipt.name, receipt.receipt, receipt.sha256
    )
}
