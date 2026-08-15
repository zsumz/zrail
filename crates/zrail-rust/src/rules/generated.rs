//! Generated Rust trees are complete, content-addressed, and visibly provenance-owned.

mod inputs;
mod lock_state;
mod manifest;
mod outputs;

use std::path::Path;

use zrail_core::{Finding, FindingSink, GeneratedSourceContract, sha256_hex};

use crate::inventory::RepositoryEntry;

use super::RuleContext;

pub(crate) use lock_state::locked_sources;

pub(super) fn evaluate(context: &RuleContext<'_>, findings: &mut FindingSink) {
    for generated in &context.contract.source.rust.generated {
        for finding in inspect_tree(
            &context.inventory.root,
            &context.inventory.entries,
            generated,
        ) {
            findings.push(finding);
        }
    }
}

fn inspect_tree(
    repository: &Path,
    entries: &[RepositoryEntry],
    generated: &GeneratedSourceContract,
) -> Vec<Finding> {
    let manifest = match manifest::read(repository, generated) {
        Ok(manifest) => manifest,
        Err(error) => return vec![manifest_finding(generated, error)],
    };
    let mut findings = Vec::new();
    if manifest.schema != 1 {
        findings.push(manifest_finding(
            generated,
            format!(
                "generated manifest schema must be 1, found {}",
                manifest.schema
            ),
        ));
    }
    if manifest.generator.trim().is_empty() {
        findings.push(manifest_finding(
            generated,
            "generated manifest requires a generator identity",
        ));
    }
    if manifest.files.is_empty() {
        findings.push(manifest_finding(
            generated,
            "generated manifest must list at least one source file",
        ));
    }
    if manifest.inputs.is_empty() {
        findings.push(manifest_finding(
            generated,
            "generated manifest must list at least one generator input",
        ));
    }
    if manifest.files.len() > manifest::MAX_FILES {
        findings.push(manifest_finding(
            generated,
            format!(
                "generated manifest exceeds the {}-file safety limit",
                manifest::MAX_FILES
            ),
        ));
        return findings;
    }
    if manifest.inputs.len() > manifest::MAX_INPUTS {
        findings.push(manifest_finding(
            generated,
            format!(
                "generated manifest exceeds the {}-input safety limit",
                manifest::MAX_INPUTS
            ),
        ));
        return findings;
    }
    inputs::compare(
        repository,
        entries,
        generated,
        &manifest.inputs,
        &mut findings,
    );
    outputs::compare(
        repository,
        entries,
        generated,
        &manifest.files,
        &mut findings,
    );
    findings
}

fn valid_digest(digest: &str) -> bool {
    digest.len() == 64
        && digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn digest(source: &str) -> String {
    sha256_hex(source.as_bytes())
}

fn manifest_finding(generated: &GeneratedSourceContract, message: impl Into<String>) -> Finding {
    let message = message.into();
    integrity_finding(generated, &generated.manifest, &message, "GEN-001")
}

fn integrity_finding(
    generated: &GeneratedSourceContract,
    path: &str,
    message: &str,
    id: &str,
) -> Finding {
    Finding::error(id, "rust.generated-source", "generated-source", message)
        .at(path, None)
        .because(&generated.reason)
        .with_help("regenerate through the declared generator; do not patch output directly")
}

#[cfg(test)]
#[path = "generated_test.rs"]
mod generated_test;
