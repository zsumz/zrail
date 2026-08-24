//! Exact mirror reachability and execution-receipt verification.

mod context;
mod findings;
mod inputs;

use std::collections::BTreeMap;

use zrail_core::{
    ExecutionReceiptStatus, FindingSink, MAX_EXECUTION_RECEIPT_BYTES, MAX_INPUT_BYTES,
    TestMirrorContract, parse_execution_receipt, read_text_with_limit,
};

use crate::{
    inventory::{FileClass, RepositoryEntry, RepositoryEntryKind},
    source::{ReachabilityKind, RustFileFacts, SyntaxGuard},
};

use super::super::RuleContext;
use findings::{mirror_finding, receipt_finding};

pub(super) fn check(
    context: &RuleContext<'_>,
    entries: &BTreeMap<&str, &RepositoryEntry>,
    findings: &mut FindingSink,
) {
    let sources = context
        .source
        .files
        .iter()
        .map(|file| (file.relative.as_str(), file))
        .collect::<BTreeMap<_, _>>();
    let mut receipt_bytes = 0usize;
    let mut input_bytes = 0usize;
    for mirror in &context.contract.source.rust.test_mirrors {
        let production = production_file(mirror, &sources, findings);
        let test = test_file(mirror, &sources, findings);
        check_declaration(mirror, test, findings);
        context::check(mirror, production, test, context, findings);
        check_receipt(
            mirror,
            production,
            test,
            entries,
            &mut receipt_bytes,
            &mut input_bytes,
            findings,
        );
    }
}

fn production_file<'a>(
    mirror: &TestMirrorContract,
    sources: &'a BTreeMap<&str, &RustFileFacts>,
    findings: &mut FindingSink,
) -> Option<&'a RustFileFacts> {
    let Some(file) = sources.get(mirror.production.as_str()).copied() else {
        findings.push(mirror_finding(
            "MIRROR-001",
            mirror,
            &mirror.production,
            "production source is missing or not analyzed",
        ));
        return None;
    };
    if !file.reachability.is_production() {
        findings.push(mirror_finding(
            "MIRROR-002",
            mirror,
            &mirror.production,
            "production source is not reachable from a Cargo production target",
        ));
    }
    Some(file)
}

fn test_file<'a>(
    mirror: &TestMirrorContract,
    sources: &'a BTreeMap<&str, &RustFileFacts>,
    findings: &mut FindingSink,
) -> Option<&'a RustFileFacts> {
    let Some(file) = sources.get(mirror.test.as_str()).copied() else {
        findings.push(mirror_finding(
            "MIRROR-003",
            mirror,
            &mirror.test,
            "test source is missing or not analyzed",
        ));
        return None;
    };
    if file.class != FileClass::Test || !file.reachability.contains(ReachabilityKind::Test) {
        findings.push(mirror_finding(
            "MIRROR-004",
            mirror,
            &mirror.test,
            "test source is not a Cargo-test-reachable test file",
        ));
    }
    Some(file)
}

fn check_declaration(
    mirror: &TestMirrorContract,
    test: Option<&RustFileFacts>,
    findings: &mut FindingSink,
) {
    let Some(test) = test else {
        return;
    };
    let matches = test
        .tests
        .iter()
        .filter(|fact| fact.name == mirror.name && fact.guard.available_in(SyntaxGuard::TestOnly))
        .count();
    if matches == 0 {
        findings.push(mirror_finding(
            "MIRROR-005",
            mirror,
            &mirror.test,
            "exact named test is not declared in the mirror file",
        ));
    } else if matches > 1 {
        findings.push(mirror_finding(
            "MIRROR-006",
            mirror,
            &mirror.test,
            "exact named test declaration is ambiguous",
        ));
    }
}

fn check_receipt(
    mirror: &TestMirrorContract,
    production: Option<&RustFileFacts>,
    test: Option<&RustFileFacts>,
    entries: &BTreeMap<&str, &RepositoryEntry>,
    receipt_bytes: &mut usize,
    input_bytes: &mut usize,
    findings: &mut FindingSink,
) {
    let Some(receipt_entry) = regular_entry(entries, &mirror.receipt) else {
        findings.push(receipt_finding(
            "RECEIPT-001",
            mirror,
            "execution receipt is missing or not a regular file",
        ));
        return;
    };
    let receipt = read_text_with_limit(&receipt_entry.absolute, MAX_INPUT_BYTES)
        .and_then(|source| {
            *receipt_bytes = receipt_bytes
                .checked_add(source.len())
                .ok_or_else(|| "execution receipt byte count overflowed".to_owned())?;
            if *receipt_bytes > MAX_EXECUTION_RECEIPT_BYTES {
                return Err(format!(
                    "execution receipts exceed {MAX_EXECUTION_RECEIPT_BYTES} total bytes"
                ));
            }
            Ok(source)
        })
        .map_err(|message| format!("cannot read execution receipt: {message}"))
        .and_then(|source| parse_execution_receipt(&source));
    let receipt = match receipt {
        Ok(receipt) => receipt,
        Err(message) => {
            findings.push(receipt_finding("RECEIPT-002", mirror, &message));
            return;
        }
    };
    if receipt.execution != mirror.execution {
        findings.push(receipt_finding(
            "RECEIPT-007",
            mirror,
            "execution receipt command, package, features, target, or toolchain differs from policy",
        ));
    }
    let Some(input_sha256) =
        inputs::digest(mirror, production, test, entries, input_bytes, findings)
    else {
        return;
    };
    if receipt.input_sha256 != input_sha256 {
        findings.push(receipt_finding(
            "RECEIPT-003",
            mirror,
            "execution receipt input digest does not match the exact reviewed execution context",
        ));
    }
    match receipt.tests.iter().find(|test| test.id == mirror.name) {
        None => findings.push(receipt_finding(
            "RECEIPT-004",
            mirror,
            "execution receipt does not report the exact named test",
        )),
        Some(test) if test.status != ExecutionReceiptStatus::Passed => {
            findings.push(receipt_finding(
                "RECEIPT-005",
                mirror,
                "execution receipt does not record the exact named test as passed",
            ));
        }
        Some(_) => {}
    }
}

fn regular_entry<'a>(
    entries: &'a BTreeMap<&str, &RepositoryEntry>,
    path: &str,
) -> Option<&'a RepositoryEntry> {
    entries
        .get(path)
        .copied()
        .filter(|entry| entry.kind == RepositoryEntryKind::File)
}
