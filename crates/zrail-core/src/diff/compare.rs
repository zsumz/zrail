//! Public semantic comparison entry point.

use crate::{Contract, LockFile};

use super::{DiffReport, contract, lock};

pub fn compare_architecture(
    before: &Contract,
    before_lock: Option<&LockFile>,
    after: &Contract,
    after_lock: Option<&LockFile>,
) -> DiffReport {
    let mut changes = contract::compare(before, after);
    changes.extend(lock::compare(before_lock, after_lock));
    DiffReport::new(changes)
}

#[cfg(test)]
#[path = "compare_test.rs"]
mod compare_test;
