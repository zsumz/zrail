//! Versioned execution receipts for exact production-to-test mirrors.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};

use crate::{TestExecutionIdentity, TestMirrorContract};

/// Execution-receipt schema supported by this crate.
pub const EXECUTION_RECEIPT_SCHEMA: u64 = 2;
/// Maximum aggregate execution-receipt bytes accepted during one repository check.
pub const MAX_EXECUTION_RECEIPT_BYTES: usize = 64 * 1024 * 1024;
/// Maximum aggregate bytes hashed as reviewed test-mirror inputs during one check.
pub const MAX_TEST_MIRROR_INPUT_BYTES: usize = 256 * 1024 * 1024;

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
    /// Exact command, package, features, target, and toolchain used by the producer.
    pub execution: TestExecutionIdentity,
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

/// Parses and validates one strict schema-2 JSON execution receipt.
pub fn parse_execution_receipt(source: &str) -> Result<ExecutionReceipt, String> {
    let receipt = serde_json::from_str::<ExecutionReceipt>(source)
        .map_err(|error| format!("invalid execution receipt JSON: {error}"))?;
    validate_execution_receipt(&receipt)?;
    Ok(receipt)
}

/// Validates receipt schema, producer version, digest, execution, and test identities.
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
    validate_execution_identity(&receipt.execution)?;
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

/// Digests exact mirror sources, reviewed inputs, and execution identity.
pub fn test_mirror_input_sha256(
    mirror: &TestMirrorContract,
    production: &[u8],
    test: &[u8],
    reviewed_inputs: &[(&str, &[u8])],
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"zrail-test-mirror-input-v2\0");
    for field in [
        mirror.production.as_bytes(),
        production,
        mirror.test.as_bytes(),
        test,
        mirror.name.as_bytes(),
    ] {
        hash_field(&mut hasher, field);
    }
    let mut inputs = reviewed_inputs.to_vec();
    inputs.sort_by_key(|(path, _)| *path);
    hash_field(&mut hasher, &(inputs.len() as u64).to_be_bytes());
    for (path, bytes) in inputs {
        hash_field(&mut hasher, path.as_bytes());
        hash_field(&mut hasher, bytes);
    }
    let execution = &mirror.execution;
    hash_field(&mut hasher, execution.command.as_bytes());
    hash_field(&mut hasher, execution.package.as_bytes());
    hash_field(&mut hasher, &[u8::from(execution.default_features)]);
    hash_field(
        &mut hasher,
        &(execution.features.len() as u64).to_be_bytes(),
    );
    for feature in &execution.features {
        hash_field(&mut hasher, feature.as_bytes());
    }
    hash_field(&mut hasher, execution.target.as_bytes());
    hash_field(&mut hasher, execution.toolchain.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn validate_execution_identity(identity: &TestExecutionIdentity) -> Result<(), String> {
    for (label, value) in [
        ("command", identity.command.as_str()),
        ("target", identity.target.as_str()),
        ("toolchain", identity.toolchain.as_str()),
    ] {
        if value.is_empty()
            || value.trim() != value
            || value
                .bytes()
                .any(|byte| matches!(byte, b'\0' | b'\r' | b'\n'))
        {
            return Err(format!(
                "execution receipt {label} must be a non-empty normalized line"
            ));
        }
    }
    if identity.package.is_empty()
        || !identity
            .package
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err("execution receipt package identity is invalid".into());
    }
    let valid_features = identity.features.iter().all(|feature| {
        !feature.is_empty()
            && feature
                .bytes()
                .all(|byte| byte.is_ascii_graphic() && !matches!(byte, b',' | b'[' | b']'))
    });
    if !valid_features || !identity.features.windows(2).all(|pair| pair[0] < pair[1]) {
        return Err("execution receipt features must be valid, unique, and sorted".into());
    }
    Ok(())
}

fn hash_field(hasher: &mut Sha256, field: &[u8]) {
    hasher.update((field.len() as u64).to_be_bytes());
    hasher.update(field);
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
