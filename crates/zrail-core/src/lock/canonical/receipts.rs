//! Validation and ordering for locked execution receipts.

use std::{ffi::OsStr, path::Path};

use super::{ensure_unique, valid_digest, valid_root};
use crate::{LockError, LockFile, versioned_producer};

pub(super) fn canonicalize(lock: &mut LockFile) -> Result<(), LockError> {
    for receipt in &lock.execution_receipts {
        validate_source_path(&receipt.production, "production")?;
        validate_source_path(&receipt.test, "test")?;
        validate_receipt_path(&receipt.receipt)?;
        if receipt.production == receipt.test {
            return Err(LockError::new(
                "locked execution receipt production and test paths must differ",
            ));
        }
        if !valid_identifier(&receipt.name) {
            return Err(LockError::new(format!(
                "locked execution receipt has invalid test name {:?}",
                receipt.name
            )));
        }
        if !valid_digest(&receipt.sha256) || !valid_digest(&receipt.input_sha256) {
            return Err(LockError::new(format!(
                "locked execution receipt {} has an invalid digest",
                receipt.receipt
            )));
        }
        if !versioned_producer(&receipt.producer) {
            return Err(LockError::new(format!(
                "locked execution receipt {} has an unversioned producer",
                receipt.receipt
            )));
        }
    }
    lock.execution_receipts
        .sort_by(|left, right| left.production.cmp(&right.production));
    ensure_unique(
        lock.execution_receipts
            .iter()
            .map(|receipt| receipt.production.as_str()),
        "locked execution receipt production",
    )?;
    ensure_unique(
        lock.execution_receipts
            .iter()
            .map(|receipt| receipt.test.as_str()),
        "locked execution receipt test",
    )?;
    ensure_unique(
        lock.execution_receipts
            .iter()
            .map(|receipt| receipt.receipt.as_str()),
        "locked execution receipt path",
    )
}

fn validate_source_path(path: &str, label: &str) -> Result<(), LockError> {
    if path == "." || !valid_root(path) || Path::new(path).extension() != Some(OsStr::new("rs")) {
        return Err(LockError::new(format!(
            "locked execution receipt {label} is not a normalized Rust file: {path}"
        )));
    }
    Ok(())
}

fn validate_receipt_path(path: &str) -> Result<(), LockError> {
    if path == "."
        || path == "zrail.lock"
        || !valid_root(path)
        || Path::new(path).extension() != Some(OsStr::new("json"))
    {
        return Err(LockError::new(format!(
            "locked execution receipt path is not a normalized JSON file: {path}"
        )));
    }
    Ok(())
}

fn valid_identifier(value: &str) -> bool {
    let value = value.strip_prefix("r#").unwrap_or(value);
    let mut bytes = value.bytes();
    bytes
        .next()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}
