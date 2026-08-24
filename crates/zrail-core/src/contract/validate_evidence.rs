//! Cross-section validation for invariant evidence and qualification gates.

mod gates;
mod graph;
mod mirrors;

use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::OsStr,
    path::Path,
};

use super::{
    Contract, EvidenceReference, GateContract, parse_evidence_reference,
    validate_limits::ValidationErrors, validate_paths::validate_repository_literal,
    validate_sets::collect_unique,
};

pub(super) fn validate(contract: &Contract, errors: &mut ValidationErrors) {
    let gates = gates::validate(contract, errors);
    graph::validate(&gates, errors);
    mirrors::validate(contract, errors);
    validate_invariants(contract, &gates, errors);
}

fn validate_invariants(
    contract: &Contract,
    gates: &BTreeMap<&str, &GateContract>,
    errors: &mut ValidationErrors,
) {
    collect_unique(
        contract
            .invariants
            .iter()
            .map(|invariant| invariant.id.as_str()),
        "invariant",
        errors,
    );
    let mut used_gates = BTreeSet::new();
    for invariant in &contract.invariants {
        if !super::evidence::valid_name(&invariant.id) {
            errors.push(format!("invalid invariant id {:?}", invariant.id));
        }
        if invariant.title.trim().is_empty() {
            errors.push(format!("invariant {:?} requires a title", invariant.id));
        }
        validate_document(contract, &invariant.id, &invariant.document, errors);
        validate_evidence(invariant, gates, &mut used_gates, errors);
    }
    let roots = used_gates.clone();
    for gate in roots {
        graph::mark_required(&gate, gates, &mut used_gates);
    }
    for gate in gates.keys().filter(|gate| !used_gates.contains(**gate)) {
        errors.push(format!(
            "gate {gate:?} is stale because no invariant evidence reaches it"
        ));
    }
}

fn validate_document(contract: &Contract, id: &str, document: &str, errors: &mut ValidationErrors) {
    let Some((path, anchor)) = document.split_once('#') else {
        errors.push(format!(
            "invariant {id:?} document must be an exact path#anchor"
        ));
        return;
    };
    validate_repository_literal(path, errors);
    if path == "." {
        errors.push(format!("invariant {id:?} document must name a file"));
    } else if gates::excluded(contract, path) {
        errors.push(format!(
            "invariant {id:?} document is hidden by repository.exclude"
        ));
    }
    if anchor.is_empty() || anchor.contains('#') {
        errors.push(format!("invariant {id:?} has an invalid document anchor"));
    }
}

fn validate_evidence(
    invariant: &super::InvariantContract,
    gates: &BTreeMap<&str, &GateContract>,
    used_gates: &mut BTreeSet<String>,
    errors: &mut ValidationErrors,
) {
    let mut values = BTreeSet::new();
    let mut has_test = false;
    let mut has_gate = false;
    for value in &invariant.evidence {
        if !values.insert(value.as_str()) {
            errors.push(format!(
                "invariant {:?} contains duplicate evidence {value:?}",
                invariant.id
            ));
            continue;
        }
        match parse_evidence_reference(value) {
            Ok(EvidenceReference::RustTest { path, .. }) => {
                has_test = true;
                validate_repository_literal(path, errors);
                if Path::new(path).extension() != Some(OsStr::new("rs")) {
                    errors.push(format!(
                        "invariant {:?} Rust test evidence must name a .rs file",
                        invariant.id
                    ));
                }
            }
            Ok(EvidenceReference::Gate { name }) => {
                has_gate = true;
                used_gates.insert(name.into());
                if !gates.contains_key(name) {
                    errors.push(format!(
                        "invariant {:?} references missing gate {name:?}",
                        invariant.id
                    ));
                }
            }
            Err(message) => errors.push(format!("invariant {:?}: {message}", invariant.id)),
        }
    }
    if !has_test {
        errors.push(format!(
            "invariant {:?} requires exact Rust test evidence",
            invariant.id
        ));
    }
    if !has_gate {
        errors.push(format!(
            "invariant {:?} requires qualification gate evidence",
            invariant.id
        ));
    }
}

#[cfg(test)]
#[path = "validate_evidence_test.rs"]
mod validate_evidence_test;
