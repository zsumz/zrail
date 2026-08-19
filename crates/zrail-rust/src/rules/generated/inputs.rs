//! Exact generator-input census and content verification.

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::{Finding, MAX_INPUT_BYTES, glob_matches, read_bytes_with_limit, sha256_hex};

use crate::inventory::{RepositoryEntry, RepositoryEntryKind, under_root};

use super::{
    GeneratedSourceContract, integrity_finding, manifest, manifest::ManifestInput,
    manifest_finding, valid_digest,
};

const MAX_TOTAL_INPUT_BYTES: usize = 128 * 1024 * 1024;

pub(super) fn compare(
    repository: &std::path::Path,
    entries: &[RepositoryEntry],
    generated: &GeneratedSourceContract,
    inputs: &[ManifestInput],
    findings: &mut Vec<Finding>,
) {
    let entries_by_path = entries
        .iter()
        .map(|entry| (entry.relative.as_str(), entry))
        .collect::<BTreeMap<_, _>>();
    let selected = selected_inputs(entries, generated, findings);
    let declared = declared_inputs(inputs, generated, findings);
    for path in selected.difference(&declared) {
        findings.push(input_finding(
            generated,
            path,
            "declared generator input is absent from its manifest",
            "GEN-005",
        ));
    }
    for path in declared.difference(&selected) {
        findings.push(input_finding(
            generated,
            path,
            "generated manifest names input outside the declared selectors",
            "GEN-005",
        ));
    }
    let mut total_bytes = 0_usize;
    for input in inputs {
        let Ok(path) = manifest::input_path(&input.path) else {
            continue;
        };
        if !check_input(
            repository,
            &entries_by_path,
            generated,
            input,
            &path,
            findings,
            &mut total_bytes,
        ) {
            break;
        }
    }
}

fn selected_inputs(
    entries: &[RepositoryEntry],
    generated: &GeneratedSourceContract,
    findings: &mut Vec<Finding>,
) -> BTreeSet<String> {
    let mut selected = BTreeSet::new();
    for pattern in &generated.inputs {
        let mut matched = false;
        for entry in entries.iter().filter(|entry| {
            entry.kind != RepositoryEntryKind::Directory && glob_matches(pattern, &entry.relative)
        }) {
            matched = true;
            if under_root(&entry.relative, &generated.root) {
                findings.push(input_finding(
                    generated,
                    &entry.relative,
                    "generated output cannot attest itself as a generator input",
                    "GEN-005",
                ));
            } else {
                selected.insert(entry.relative.clone());
            }
        }
        if !matched {
            findings.push(input_finding(
                generated,
                &generated.manifest,
                &format!("generator input selector {pattern:?} matches no repository file"),
                "GEN-005",
            ));
        }
    }
    selected
}

fn declared_inputs(
    inputs: &[ManifestInput],
    generated: &GeneratedSourceContract,
    findings: &mut Vec<Finding>,
) -> BTreeSet<String> {
    let mut declared = BTreeSet::new();
    for input in inputs {
        let path = match manifest::input_path(&input.path) {
            Ok(path) => path,
            Err(error) => {
                findings.push(manifest_finding(generated, error));
                continue;
            }
        };
        if !declared.insert(path.clone()) {
            findings.push(manifest_finding(
                generated,
                format!("generated manifest contains duplicate input path {path:?}"),
            ));
        }
    }
    declared
}

fn check_input(
    repository: &std::path::Path,
    entries: &BTreeMap<&str, &RepositoryEntry>,
    generated: &GeneratedSourceContract,
    input: &ManifestInput,
    path: &str,
    findings: &mut Vec<Finding>,
    total_bytes: &mut usize,
) -> bool {
    if !valid_digest(&input.sha256) {
        findings.push(input_finding(
            generated,
            path,
            "generator input has an invalid SHA-256 digest",
            "GEN-004",
        ));
        return true;
    }
    if !matches!(entries.get(path), Some(entry) if entry.kind == RepositoryEntryKind::File) {
        findings.push(input_finding(
            generated,
            path,
            "generator input must be a regular repository file",
            "GEN-004",
        ));
        return true;
    }
    let bytes = match read_bytes_with_limit(&repository.join(path), MAX_INPUT_BYTES) {
        Ok(bytes) => bytes,
        Err(error) => {
            findings.push(input_finding(generated, path, &error, "GEN-004"));
            return true;
        }
    };
    *total_bytes = match total_bytes.checked_add(bytes.len()) {
        Some(total) if total <= MAX_TOTAL_INPUT_BYTES => total,
        _ => {
            findings.push(input_finding(
                generated,
                path,
                &format!(
                    "generator inputs exceed the {MAX_TOTAL_INPUT_BYTES}-byte total safety limit"
                ),
                "GEN-004",
            ));
            return false;
        }
    };
    if sha256_hex(&bytes) != input.sha256 {
        findings.push(input_finding(
            generated,
            path,
            "generator input hash differs from its manifest",
            "GEN-004",
        ));
    }
    true
}

fn input_finding(
    generated: &GeneratedSourceContract,
    path: &str,
    message: &str,
    id: &str,
) -> Finding {
    integrity_finding(generated, path, message, id).with_help(
        "regenerate the snapshot and its complete input census through the declared generator",
    )
}
