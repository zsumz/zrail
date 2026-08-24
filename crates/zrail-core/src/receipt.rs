//! Versioned execution receipts for exact production-to-test mirrors.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};

/// Execution-receipt schema supported by this crate.
pub const EXECUTION_RECEIPT_SCHEMA: u64 = 1;
/// Maximum aggregate execution-receipt bytes accepted during one repository check.
pub const MAX_EXECUTION_RECEIPT_BYTES: usize = 64 * 1024 * 1024;

/// A producer-authored statement that exact tests ran against exact inputs.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionReceipt {
    /// Receipt format version.
    pub schema: u64,
    /// Producer identity and semantic version, formatted as `name major.minor.patch`.
    pub producer: String,
    /// Deterministic digest returned by [`test_mirror_input_sha256`].
    pub input_sha256: String,
    /// Optional toolchain identity reported by the producer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub toolchain: Option<String>,
    /// Exact test outcomes recorded by the producer.
    pub tests: Vec<ExecutionReceiptTest>,
}

/// One named test outcome in an [`ExecutionReceipt`].
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionReceiptTest {
    /// Exact Rust test identifier.
    pub id: String,
    /// Observed execution outcome.
    pub status: ExecutionReceiptStatus,
}

/// Outcomes a receipt producer may report.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutionReceiptStatus {
    /// The test completed successfully.
    Passed,
    /// The test ran and failed.
    Failed,
    /// The producer did not execute the test.
    Skipped,
}

/// Parses and validates one strict schema-1 JSON execution receipt.
pub fn parse_execution_receipt(source: &str) -> Result<ExecutionReceipt, String> {
    let receipt = serde_json::from_str::<ExecutionReceipt>(source)
        .map_err(|error| format!("invalid execution receipt JSON: {error}"))?;
    validate_execution_receipt(&receipt)?;
    Ok(receipt)
}

/// Validates receipt schema, producer version, digest, toolchain, and test identities.
pub fn validate_execution_receipt(receipt: &ExecutionReceipt) -> Result<(), String> {
    if receipt.schema != EXECUTION_RECEIPT_SCHEMA {
        return Err(format!(
            "unsupported execution receipt schema {}; expected {EXECUTION_RECEIPT_SCHEMA}",
            receipt.schema
        ));
    }
    if !versioned_producer(&receipt.producer) {
        return Err(
            "execution receipt producer must be `name major.minor.patch` with a version".into(),
        );
    }
    if !valid_digest(&receipt.input_sha256) {
        return Err(
            "execution receipt input_sha256 must be 64 lowercase hexadecimal characters".into(),
        );
    }
    if receipt
        .toolchain
        .as_ref()
        .is_some_and(|toolchain| toolchain.trim().is_empty())
    {
        return Err("execution receipt toolchain may not be empty".into());
    }
    if receipt.tests.is_empty() {
        return Err("execution receipt must report at least one test".into());
    }
    let mut tests = BTreeSet::new();
    for test in &receipt.tests {
        if !valid_identifier(&test.id) {
            return Err(format!(
                "execution receipt test id is not an exact Rust identifier: {:?}",
                test.id
            ));
        }
        if !tests.insert(test.id.as_str()) {
            return Err(format!(
                "execution receipt contains duplicate test id {:?}",
                test.id
            ));
        }
    }
    Ok(())
}

/// Returns whether `producer` includes a non-empty name and semantic version.
pub fn versioned_producer(producer: &str) -> bool {
    let Some((name, version)) = producer.rsplit_once(' ') else {
        return false;
    };
    !name.trim().is_empty() && valid_version(version)
}

/// Digests an exact mirror identity and both source byte streams with unambiguous framing.
pub fn test_mirror_input_sha256(
    production_path: &str,
    production: &[u8],
    test_path: &str,
    test: &[u8],
    test_name: &str,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"zrail-test-mirror-input-v1\0");
    for field in [
        production_path.as_bytes(),
        production,
        test_path.as_bytes(),
        test,
        test_name.as_bytes(),
    ] {
        hasher.update((field.len() as u64).to_be_bytes());
        hasher.update(field);
    }
    format!("{:x}", hasher.finalize())
}

fn valid_version(version: &str) -> bool {
    let core_end = version.find(['-', '+']).unwrap_or(version.len());
    let core = &version[..core_end];
    let mut parts = core.split('.');
    let core_valid = (0..3).all(|_| {
        parts
            .next()
            .is_some_and(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
    }) && parts.next().is_none();
    core_valid
        && version[core_end..]
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'+' | b'.'))
}

fn valid_identifier(value: &str) -> bool {
    let value = value.strip_prefix("r#").unwrap_or(value);
    let mut bytes = value.bytes();
    bytes
        .next()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn valid_digest(digest: &str) -> bool {
    digest.len() == 64
        && digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(test)]
#[path = "receipt_test.rs"]
mod receipt_test;
