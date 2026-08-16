//! Gate paths, content inputs, and qualification dependencies are exact and acyclic.

use std::collections::{BTreeMap, BTreeSet};

use crate::path::glob_matches;

use super::super::{
    Contract, GateContract, GateKind,
    validate_limits::ValidationErrors,
    validate_paths::validate_repository_literal,
    validate_sets::{collect_unique, require_reason},
};

pub(super) fn validate<'a>(
    contract: &'a Contract,
    errors: &mut ValidationErrors,
) -> BTreeMap<&'a str, &'a GateContract> {
    let names = collect_unique(
        contract.gates.iter().map(|gate| gate.name.as_str()),
        "gate",
        errors,
    );
    let mut paths = BTreeSet::new();
    for gate in &contract.gates {
        validate_gate(contract, gate, &names, &mut paths, errors);
    }
    contract
        .gates
        .iter()
        .map(|gate| (gate.name.as_str(), gate))
        .collect()
}

fn validate_gate(
    contract: &Contract,
    gate: &GateContract,
    names: &BTreeSet<&str>,
    paths: &mut BTreeSet<String>,
    errors: &mut ValidationErrors,
) {
    if !super::super::evidence::valid_name(&gate.name) {
        errors.push(format!("invalid gate name {:?}", gate.name));
    }
    validate_gate_path(contract, gate, &gate.path, "file", errors);
    if !paths.insert(gate.path.clone()) {
        errors.push(format!("multiple gates attest path {:?}", gate.path));
    }
    validate_inputs(contract, gate, errors);
    require_reason("gate", &gate.name, &gate.reason, errors);
    validate_requirements(gate, names, errors);
    if gate.kind != GateKind::Local && gate.requires.is_empty() {
        errors.push(format!(
            "{:?} gate {:?} must require a lower qualification gate",
            gate.kind, gate.name
        ));
    }
}

fn validate_inputs(contract: &Contract, gate: &GateContract, errors: &mut ValidationErrors) {
    let mut inputs = BTreeSet::new();
    for input in &gate.inputs {
        validate_gate_path(contract, gate, input, "input", errors);
        if input == &gate.path {
            errors.push(format!(
                "gate {:?} repeats its primary path as an input",
                gate.name
            ));
        }
        if !inputs.insert(input.as_str()) {
            errors.push(format!(
                "gate {:?} contains duplicate input {input:?}",
                gate.name
            ));
        }
    }
}

fn validate_gate_path(
    contract: &Contract,
    gate: &GateContract,
    path: &str,
    label: &str,
    errors: &mut ValidationErrors,
) {
    validate_repository_literal(path, errors);
    if path == "." {
        errors.push(format!("gate {:?} {label} must name a file", gate.name));
    } else if path == "zrail.lock" {
        errors.push(format!(
            "zrail.lock cannot attest its own contents as gate {:?} {label}",
            gate.name
        ));
    }
    if excluded(contract, path) {
        errors.push(format!(
            "gate {:?} {label} is hidden by repository.exclude: {path:?}",
            gate.name
        ));
    }
}

fn validate_requirements(
    gate: &GateContract,
    names: &BTreeSet<&str>,
    errors: &mut ValidationErrors,
) {
    let mut requirements = BTreeSet::new();
    for required in &gate.requires {
        if !requirements.insert(required.as_str()) {
            errors.push(format!(
                "gate {:?} contains duplicate requirement {required:?}",
                gate.name
            ));
        }
        if required == &gate.name {
            errors.push(format!("gate {:?} may not require itself", gate.name));
        } else if !names.contains(required.as_str()) {
            errors.push(format!(
                "gate {:?} requires missing gate {required:?}",
                gate.name
            ));
        }
    }
}

pub(super) fn excluded(contract: &Contract, path: &str) -> bool {
    contract
        .repository
        .exclude
        .iter()
        .any(|pattern| glob_matches(pattern, path) || path.starts_with(&format!("{pattern}/")))
}
