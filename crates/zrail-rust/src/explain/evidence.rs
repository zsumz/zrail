//! Invariant relationships included in path-scoped explanations.

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
                Ok(EvidenceReference::Gate { name }) => gates
                    .iter()
                    .any(|gate| gate.name == name && gate.path == path),
                Err(_) => false,
            })
}

#[cfg(test)]
#[path = "evidence_test.rs"]
mod evidence_test;
