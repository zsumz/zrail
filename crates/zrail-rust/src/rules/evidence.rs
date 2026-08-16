//! Live exact evidence and reviewed qualification-gate inputs.

mod gates;

use std::collections::BTreeMap;

use zrail_core::{
    EvidenceReference, Finding, FindingSink,
    input::{MAX_INPUT_BYTES, read_text_with_limit},
    parse_evidence_reference,
};

use crate::inventory::{FileClass, RepositoryEntry, RepositoryEntryKind};

use super::RuleContext;

pub(super) fn evaluate(context: &RuleContext<'_>, findings: &mut FindingSink) {
    let entries = context
        .inventory
        .entries
        .iter()
        .map(|entry| (entry.relative.as_str(), entry))
        .collect::<BTreeMap<_, _>>();
    gates::check(context, &entries, findings);
    for invariant in &context.contract.invariants {
        check_document(&invariant.id, &invariant.document, &entries, findings);
        for evidence in &invariant.evidence {
            if let Ok(EvidenceReference::RustTest { path, test }) =
                parse_evidence_reference(evidence)
            {
                check_test(context, &invariant.id, path, test, findings);
            }
        }
    }
}

fn check_document(
    invariant: &str,
    document: &str,
    entries: &BTreeMap<&str, &RepositoryEntry>,
    findings: &mut FindingSink,
) {
    let Some((path, anchor)) = document.split_once('#') else {
        return;
    };
    let Some(entry) = entries.get(path) else {
        findings.push(evidence_finding(
            "EVID-001",
            invariant,
            path,
            format!("invariant document {path:?} is missing"),
        ));
        return;
    };
    if entry.kind != RepositoryEntryKind::File {
        findings.push(evidence_finding(
            "EVID-001",
            invariant,
            path,
            format!("invariant document {path:?} is not a regular file"),
        ));
        return;
    }
    match read_text_with_limit(&entry.absolute, MAX_INPUT_BYTES) {
        Ok(source) if !contains_anchor(&source, anchor) => findings.push(evidence_finding(
            "EVID-002",
            invariant,
            path,
            format!("invariant document {path:?} has no anchor #{anchor}"),
        )),
        Err(message) => findings.push(evidence_finding("EVID-001", invariant, path, message)),
        _ => {}
    }
}

fn check_test(
    context: &RuleContext<'_>,
    invariant: &str,
    path: &str,
    test: &str,
    findings: &mut FindingSink,
) {
    let Some(file) = context
        .source
        .files
        .iter()
        .find(|file| file.relative == path)
    else {
        findings.push(evidence_finding(
            "EVID-003",
            invariant,
            path,
            format!("Rust test evidence {path}::{test} is missing"),
        ));
        return;
    };
    if file.class != FileClass::Test {
        findings.push(evidence_finding(
            "EVID-003",
            invariant,
            path,
            format!("Rust test evidence {path}::{test} is not in test source"),
        ));
        return;
    }
    let matches = file.tests.iter().filter(|fact| fact.name == test).count();
    if matches == 0 {
        findings.push(evidence_finding(
            "EVID-003",
            invariant,
            path,
            format!("Rust test evidence {path}::{test} is not declared"),
        ));
    } else if matches > 1 {
        findings.push(evidence_finding(
            "EVID-004",
            invariant,
            path,
            format!("Rust test evidence {path}::{test} is ambiguous"),
        ));
    }
}

fn evidence_finding(id: &str, invariant: &str, path: &str, message: String) -> Finding {
    Finding::error(id, "invariant.evidence", invariant, message).at(path, None)
}

fn contains_anchor(source: &str, anchor: &str) -> bool {
    let mut fence = None;
    let double_quoted = format!("<a id=\"{anchor}\"></a>");
    let single_quoted = format!("<a id='{anchor}'></a>");
    for line in source.lines() {
        let trimmed = line.trim();
        if let Some(marker) = fence {
            if fenced(trimmed, marker) {
                fence = None;
            }
            continue;
        }
        if let Some(marker) = fence_marker(trimmed) {
            fence = Some(marker);
            continue;
        }
        if trimmed == double_quoted || trimmed == single_quoted {
            return true;
        }
        if let Some(heading) = markdown_heading(line)
            && (slug(&heading) == anchor
                || heading
                    .rsplit_once(" {#")
                    .and_then(|(_, suffix)| suffix.strip_suffix('}'))
                    == Some(anchor))
        {
            return true;
        }
    }
    false
}

fn fence_marker(line: &str) -> Option<char> {
    ['`', '~'].into_iter().find(|marker| fenced(line, *marker))
}

fn fenced(line: &str, marker: char) -> bool {
    line.chars()
        .take_while(|character| *character == marker)
        .count()
        >= 3
}

fn markdown_heading(line: &str) -> Option<String> {
    let line = line.trim_start();
    let heading = line.trim_start_matches('#');
    (heading.len() < line.len() && heading.starts_with(' '))
        .then(|| heading.trim().trim_end_matches('#').trim().to_owned())
}

fn slug(heading: &str) -> String {
    let mut output = String::new();
    let mut separator = false;
    for character in heading.chars() {
        if character.is_alphanumeric() || character == '_' {
            if separator && !output.is_empty() {
                output.push('-');
            }
            output.extend(character.to_lowercase());
            separator = false;
        } else if character.is_whitespace() || character == '-' {
            separator = true;
        }
    }
    output
}

#[cfg(test)]
#[path = "evidence_test.rs"]
mod evidence_test;
