//! Candidate lock state for parseable execution receipts.

use std::collections::BTreeMap;

use zrail_core::{
    LockedExecutionReceipt, MAX_EXECUTION_RECEIPT_BYTES, MAX_INPUT_BYTES, parse_execution_receipt,
    read_bytes_with_limit, sha256_hex,
};

use crate::inventory::{RepositoryEntry, RepositoryEntryKind};

use super::model::RepositoryModel;

pub(super) fn locked(model: &RepositoryModel) -> Vec<LockedExecutionReceipt> {
    let entries = model
        .inventory
        .entries
        .iter()
        .map(|entry| (entry.relative.as_str(), entry))
        .collect::<BTreeMap<_, _>>();
    let mut bytes = 0usize;
    let mut receipts = Vec::new();
    for mirror in &model.bundle.contract.source.rust.test_mirrors {
        let Some((receipt, length)) = locked_receipt(mirror, &entries) else {
            continue;
        };
        let Some(total) = bytes.checked_add(length) else {
            break;
        };
        if total > MAX_EXECUTION_RECEIPT_BYTES {
            break;
        }
        bytes = total;
        receipts.push(receipt);
    }
    receipts
}

fn locked_receipt(
    mirror: &zrail_core::TestMirrorContract,
    entries: &BTreeMap<&str, &RepositoryEntry>,
) -> Option<(LockedExecutionReceipt, usize)> {
    let entry = entries
        .get(mirror.receipt.as_str())
        .copied()
        .filter(|entry| entry.kind == RepositoryEntryKind::File)?;
    let bytes = read_bytes_with_limit(&entry.absolute, MAX_INPUT_BYTES).ok()?;
    let source = std::str::from_utf8(&bytes).ok()?;
    let receipt = parse_execution_receipt(source).ok()?;
    let length = bytes.len();
    Some((
        LockedExecutionReceipt {
            production: mirror.production.clone(),
            test: mirror.test.clone(),
            name: mirror.name.clone(),
            receipt: mirror.receipt.clone(),
            sha256: sha256_hex(&bytes),
            input_sha256: receipt.input_sha256,
            producer: receipt.producer,
        },
        length,
    ))
}
