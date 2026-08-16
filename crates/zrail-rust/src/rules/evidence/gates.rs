//! Qualification gate executables and every declared behavioral input must exist as files.

use std::collections::BTreeMap;

use zrail_core::{Finding, FindingSink};

use crate::inventory::{RepositoryEntry, RepositoryEntryKind};

use super::super::RuleContext;

pub(super) fn check(
    context: &RuleContext<'_>,
    entries: &BTreeMap<&str, &RepositoryEntry>,
    findings: &mut FindingSink,
) {
    for gate in &context.contract.gates {
        check_path(
            entries,
            findings,
            &gate.name,
            &gate.path,
            "QUAL-001",
            "QUAL-002",
            "qualification gate file",
        );
        for input in &gate.inputs {
            check_path(
                entries,
                findings,
                &gate.name,
                input,
                "QUAL-003",
                "QUAL-004",
                "qualification gate input",
            );
        }
    }
}

fn check_path(
    entries: &BTreeMap<&str, &RepositoryEntry>,
    findings: &mut FindingSink,
    gate: &str,
    path: &str,
    missing: &str,
    non_file: &str,
    label: &str,
) {
    let Some(entry) = entries.get(path) else {
        findings.push(
            Finding::error(
                missing,
                "qualification.gate",
                gate,
                format!("{label} {path:?} is missing"),
            )
            .at(path, None),
        );
        return;
    };
    if entry.kind != RepositoryEntryKind::File {
        findings.push(
            Finding::error(
                non_file,
                "qualification.gate",
                gate,
                format!("{label} {path:?} is not a regular repository file"),
            )
            .at(path, None),
        );
    }
}
