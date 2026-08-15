//! Exact generated-output census, banners, and content verification.

use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use zrail_core::{Finding, GeneratedSourceContract, input::read_text_with_limit};

use crate::inventory::{RepositoryEntry, RepositoryEntryKind, under_root};

use super::{
    digest, integrity_finding, manifest, manifest::ManifestFile, manifest_finding, valid_digest,
};

pub(super) fn compare(
    repository: &Path,
    entries: &[RepositoryEntry],
    generated: &GeneratedSourceContract,
    files: &[ManifestFile],
    findings: &mut Vec<Finding>,
) {
    let entries = entries
        .iter()
        .map(|entry| (entry.relative.as_str(), entry))
        .collect::<BTreeMap<_, _>>();
    let actual = entries
        .keys()
        .filter(|path| under_root(path, &generated.root) && manifest::source_candidate(path))
        .map(|path| (*path).to_owned())
        .collect::<BTreeSet<_>>();
    let mut expected = BTreeSet::new();
    for file in files {
        let path = match manifest::file_path(&generated.root, &file.path) {
            Ok(path) => path,
            Err(error) => {
                findings.push(manifest_finding(generated, error));
                continue;
            }
        };
        if !expected.insert(path.clone()) {
            findings.push(manifest_finding(
                generated,
                format!("generated manifest contains duplicate path {path:?}"),
            ));
            continue;
        }
        check_file(repository, &entries, generated, file, &path, findings);
    }
    check_auxiliary(generated, &expected, findings);
    compare_census(generated, &actual, &expected, findings);
}

fn check_auxiliary(
    generated: &GeneratedSourceContract,
    expected: &BTreeSet<String>,
    findings: &mut Vec<Finding>,
) {
    for auxiliary in &generated.auxiliary {
        let Ok(path) = manifest::file_path(&generated.root, auxiliary) else {
            continue;
        };
        if !expected.contains(&path) {
            findings.push(manifest_finding(
                generated,
                format!("generated auxiliary source {path:?} is absent from its manifest"),
            ));
        }
    }
}

fn compare_census(
    generated: &GeneratedSourceContract,
    actual: &BTreeSet<String>,
    expected: &BTreeSet<String>,
    findings: &mut Vec<Finding>,
) {
    for path in actual.difference(expected) {
        findings.push(integrity_finding(
            generated,
            path,
            "generated Rust source is absent from its manifest",
            "GEN-003",
        ));
    }
    for path in expected.difference(actual) {
        findings.push(integrity_finding(
            generated,
            path,
            "generated manifest names missing Rust source",
            "GEN-003",
        ));
    }
}

fn check_file(
    repository: &Path,
    entries: &BTreeMap<&str, &RepositoryEntry>,
    generated: &GeneratedSourceContract,
    file: &ManifestFile,
    path: &str,
    findings: &mut Vec<Finding>,
) {
    if !valid_digest(&file.sha256) {
        findings.push(manifest_finding(
            generated,
            format!("generated file {path:?} has an invalid SHA-256 digest"),
        ));
        return;
    }
    if !matches!(entries.get(path), Some(entry) if entry.kind == RepositoryEntryKind::File) {
        return;
    }
    let source = match read_text_with_limit(&repository.join(path), manifest::MAX_SOURCE_BYTES) {
        Ok(source) => source,
        Err(error) => {
            findings.push(integrity_finding(generated, path, &error, "GEN-002"));
            return;
        }
    };
    let provenance = manifest::banner(path).unwrap_or("//! @generated");
    if !source.trim_start().starts_with(provenance) {
        findings.push(integrity_finding(
            generated,
            path,
            &format!("generated source must begin with `{provenance}`"),
            "GEN-002",
        ));
    }
    if digest(&source) != file.sha256 {
        findings.push(integrity_finding(
            generated,
            path,
            "generated source hash differs from its manifest",
            "GEN-002",
        ));
    }
}
