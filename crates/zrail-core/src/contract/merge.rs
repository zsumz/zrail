//! Conflict-free merging for local contract fragments.

use std::collections::BTreeMap;

use super::{
    AnalysisContract, Contract, DependenciesContract, DependencyRule, GateContract,
    InvariantContract, LayerContract, OwnerContract, ProfileContract, RatchetContract,
    RepositoryContract, ScopeContract, SourceContract,
    load::{ContractError, ContractFile},
};

#[derive(Debug, Default)]
pub(super) struct MergeState {
    schema: Option<u64>,
    adapters: Option<Vec<String>>,
    repository: Option<RepositoryContract>,
    dependencies: Option<DependenciesContract>,
    source: Option<SourceContract>,
    analysis: Option<AnalysisContract>,
    profiles: BTreeMap<String, ProfileContract>,
    layers: Vec<LayerContract>,
    dependency_rules: Vec<DependencyRule>,
    scopes: Vec<ScopeContract>,
    owners: Vec<OwnerContract>,
    ratchets: Vec<RatchetContract>,
    gates: Vec<GateContract>,
    invariants: Vec<InvariantContract>,
}

impl MergeState {
    pub(super) fn merge(&mut self, file: ContractFile, origin: &str) -> Result<(), ContractError> {
        merge_singleton(&mut self.schema, file.schema, "schema", origin)?;
        merge_singleton(&mut self.adapters, file.adapters, "adapters", origin)?;
        merge_singleton(&mut self.repository, file.repository, "repository", origin)?;
        merge_singleton(
            &mut self.dependencies,
            file.dependencies,
            "dependencies",
            origin,
        )?;
        merge_singleton(&mut self.source, file.source, "source", origin)?;
        merge_singleton(&mut self.analysis, file.analysis, "analysis", origin)?;
        for (name, profile) in file.profiles {
            if self.profiles.insert(name.clone(), profile).is_some() {
                return Err(ContractError::one(format!(
                    "duplicate profile {name:?} while loading {origin}"
                )));
            }
        }
        self.layers.extend(file.layers);
        self.dependency_rules.extend(file.dependency_rules);
        self.scopes.extend(file.scopes);
        self.owners.extend(file.owners);
        self.ratchets.extend(file.ratchets);
        self.gates.extend(file.gates);
        self.invariants.extend(file.invariants);
        Ok(())
    }

    pub(super) fn finish(self) -> Result<Contract, ContractError> {
        let mut missing = Vec::new();
        if self.schema.is_none() {
            missing.push("schema");
        }
        if self.adapters.is_none() {
            missing.push("adapters");
        }
        if self.repository.is_none() {
            missing.push("repository");
        }
        if self.dependencies.is_none() {
            missing.push("dependencies");
        }
        if self.source.is_none() {
            missing.push("source");
        }
        if !missing.is_empty() {
            return Err(ContractError::one(format!(
                "contract is missing required sections: {}",
                missing.join(", ")
            )));
        }
        Ok(Contract {
            schema: required(self.schema, "schema")?,
            adapters: required(self.adapters, "adapters")?,
            repository: required(self.repository, "repository")?,
            dependencies: required(self.dependencies, "dependencies")?,
            source: required(self.source, "source")?,
            analysis: self.analysis.unwrap_or_default(),
            profiles: self.profiles,
            layers: self.layers,
            dependency_rules: self.dependency_rules,
            scopes: self.scopes,
            owners: self.owners,
            ratchets: self.ratchets,
            gates: self.gates,
            invariants: self.invariants,
        })
    }
}

fn merge_singleton<T>(
    target: &mut Option<T>,
    value: Option<T>,
    label: &str,
    origin: &str,
) -> Result<(), ContractError> {
    let Some(value) = value else {
        return Ok(());
    };
    if target.is_some() {
        return Err(ContractError::one(format!(
            "singleton section {label:?} is declared more than once; conflict at {origin}"
        )));
    }
    *target = Some(value);
    Ok(())
}

fn required<T>(value: Option<T>, label: &str) -> Result<T, ContractError> {
    value.ok_or_else(|| ContractError::one(format!("missing required section {label}")))
}
