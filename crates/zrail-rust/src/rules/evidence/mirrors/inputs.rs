//! Bounded reading and hashing of exact mirror execution inputs.

use std::collections::BTreeMap;

use zrail_core::{
    FindingSink, MAX_INPUT_BYTES, MAX_TEST_MIRROR_INPUT_BYTES, TestMirrorContract,
    read_bytes_with_limit, test_mirror_input_sha256,
};

use crate::{
    inventory::{RepositoryEntry, RepositoryEntryKind},
    source::RustFileFacts,
};

use super::findings::receipt_finding;

pub(super) fn digest(
    mirror: &TestMirrorContract,
    production: Option<&RustFileFacts>,
    test: Option<&RustFileFacts>,
    entries: &BTreeMap<&str, &RepositoryEntry>,
    aggregate_bytes: &mut usize,
    findings: &mut FindingSink,
) -> Option<String> {
    production?;
    test?;
    let result = (|| {
        let production = read_input(entries, &mirror.production, aggregate_bytes)?;
        let test = read_input(entries, &mirror.test, aggregate_bytes)?;
        let mut owned = Vec::with_capacity(mirror.inputs.len());
        for path in &mirror.inputs {
            owned.push((path.as_str(), read_input(entries, path, aggregate_bytes)?));
        }
        let reviewed = owned
            .iter()
            .map(|(path, bytes)| (*path, bytes.as_slice()))
            .collect::<Vec<_>>();
        Ok::<_, String>(test_mirror_input_sha256(
            mirror,
            &production,
            &test,
            &reviewed,
        ))
    })();
    match result {
        Ok(digest) => Some(digest),
        Err(message) => {
            findings.push(receipt_finding("RECEIPT-006", mirror, &message));
            None
        }
    }
}

fn read_input(
    entries: &BTreeMap<&str, &RepositoryEntry>,
    path: &str,
    aggregate_bytes: &mut usize,
) -> Result<Vec<u8>, String> {
    let entry = entries
        .get(path)
        .copied()
        .filter(|entry| entry.kind == RepositoryEntryKind::File)
        .ok_or_else(|| format!("mirror input {path:?} is missing or not a regular file"))?;
    let bytes = read_bytes_with_limit(&entry.absolute, MAX_INPUT_BYTES)?;
    *aggregate_bytes = aggregate_bytes
        .checked_add(bytes.len())
        .ok_or_else(|| "test mirror input byte count overflowed".to_owned())?;
    if *aggregate_bytes > MAX_TEST_MIRROR_INPUT_BYTES {
        return Err(format!(
            "test mirror inputs exceed {MAX_TEST_MIRROR_INPUT_BYTES} total bytes"
        ));
    }
    Ok(bytes)
}
