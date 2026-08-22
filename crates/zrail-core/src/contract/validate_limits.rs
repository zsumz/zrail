//! Contract size and error ceilings keep invalid policy deterministic and bounded.

use super::{Contract, ContractError};

const MAX_CONTRACT_ITEMS: usize = 50_000;
const MAX_VALIDATION_ERRORS: usize = 256;

pub(super) struct ValidationErrors {
    messages: Vec<String>,
    omitted: usize,
}

impl ValidationErrors {
    pub(super) fn new() -> Self {
        Self {
            messages: Vec::new(),
            omitted: 0,
        }
    }

    pub(super) fn push(&mut self, message: String) {
        if self.messages.len() < MAX_VALIDATION_ERRORS - 1 {
            self.messages.push(message);
        } else {
            self.omitted += 1;
        }
    }

    pub(super) fn is_empty(&self) -> bool {
        self.messages.is_empty() && self.omitted == 0
    }

    pub(super) fn finish(mut self) -> Vec<String> {
        if self.omitted > 0 {
            self.messages.push(format!(
                "contract validation stopped reporting after {} errors; {} additional errors omitted",
                MAX_VALIDATION_ERRORS - 1,
                self.omitted
            ));
        }
        self.messages
    }
}

pub(super) fn enforce_contract_size(contract: &Contract) -> Result<(), ContractError> {
    if contract_items(contract) <= MAX_CONTRACT_ITEMS {
        return Ok(());
    }
    Err(ContractError::one(format!(
        "contract exceeds the {MAX_CONTRACT_ITEMS}-item safety limit"
    )))
}

fn contract_items(contract: &Contract) -> usize {
    let mut count = contract.adapters.len()
        + contract.repository.roots.len()
        + contract.repository.exclude.len()
        + contract.profiles.len()
        + contract.layers.len()
        + contract.dependency_rules.len()
        + contract.scopes.len()
        + contract.owners.len()
        + contract.ratchets.len()
        + contract.dependencies.crate_roots.len()
        + contract.source.rust.file_roles.len()
        + contract.source.rust.generated.len()
        + contract.source.rust.out_dir.len()
        + contract.source.rust.item_macros.len();
    count += contract.source.rust.macros.allow.len();
    count += contract
        .source
        .rust
        .generated
        .iter()
        .map(|generated| generated.inputs.len() + generated.auxiliary.len())
        .sum::<usize>();
    count += contract
        .profiles
        .values()
        .map(|profile| profile.effects.deny.len())
        .sum::<usize>();
    count += contract
        .layers
        .iter()
        .map(|layer| layer.packages.len() + layer.may_depend_on.len() + layer.profiles.len())
        .sum::<usize>();
    count += contract
        .dependency_rules
        .iter()
        .map(|rule| rule.deny.len())
        .sum::<usize>();
    count += contract
        .scopes
        .iter()
        .map(|scope| scope.include.len() + scope.exclude.len() + scope.symbols.deny.len())
        .sum::<usize>();
    count += contract
        .owners
        .iter()
        .map(|owner| owner.allow.len() + owner.within.len())
        .sum::<usize>();
    count += evidence_items(&contract.gates, &contract.invariants);
    count
}

fn evidence_items(gates: &[super::GateContract], invariants: &[super::InvariantContract]) -> usize {
    gates.len()
        + gates
            .iter()
            .map(|gate| gate.inputs.len() + gate.requires.len())
            .sum::<usize>()
        + invariants.len()
        + invariants
            .iter()
            .map(|invariant| invariant.evidence.len())
            .sum::<usize>()
}

#[cfg(test)]
#[path = "validate_limits_test.rs"]
mod validate_limits_test;
