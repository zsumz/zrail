//! Invariant relationships included in path-scoped explanations.

use std::collections::BTreeSet;

use zrail_core::{
    Contract, EvidenceReference, GateContract, InvariantContract, parse_evidence_reference,
};

pub(super) fn for_path(contract: &Contract, path: &str) -> Vec<String> {
    contract
        .invariants
        .iter()
        .filter(|invariant| mentions_path(&contract.gates, invariant, path))
        .map(|invariant| invariant.id.clone())
        .collect()
}

fn mentions_path(gates: &[GateContract], invariant: &InvariantContract, path: &str) -> bool {
    invariant
        .document
        .split_once('#')
        .is_some_and(|(document, _)| document == path)
        || invariant
            .evidence
            .iter()
            .any(|evidence| match parse_evidence_reference(evidence) {
                Ok(EvidenceReference::RustTest {
                    path: test_path, ..
                }) => test_path == path,
                Ok(EvidenceReference::Gate { name }) => {
                    gate_mentions_path(gates, name, path, &mut BTreeSet::new())
                }
                Err(_) => false,
            })
}

fn gate_mentions_path(
    gates: &[GateContract],
    name: &str,
    path: &str,
    seen: &mut BTreeSet<String>,
) -> bool {
    if !seen.insert(name.into()) {
        return false;
    }
    let Some(gate) = gates.iter().find(|gate| gate.name == name) else {
        return false;
    };
    gate.path == path
        || gate.inputs.iter().any(|input| input == path)
        || gate
            .requires
            .iter()
            .any(|required| gate_mentions_path(gates, required, path, seen))
}

#[cfg(test)]
#[path = "evidence_test.rs"]
mod evidence_test;
