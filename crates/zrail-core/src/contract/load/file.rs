//! Strict shape of one root or imported contract fragment.

use std::collections::BTreeMap;

use serde::Deserialize;

use super::super::{
    AnalysisContract, DependenciesContract, DependencyRule, GateContract, InvariantContract,
    LayerContract, OwnerContract, ProfileContract, RatchetContract, RepositoryContract,
    ScopeContract, SourceContract,
};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::contract) struct ContractFile {
    pub(in crate::contract) schema: Option<u64>,
    pub(in crate::contract) adapters: Option<Vec<String>>,
    #[serde(default)]
    pub(in crate::contract) imports: Vec<String>,
    pub(in crate::contract) repository: Option<RepositoryContract>,
    pub(in crate::contract) dependencies: Option<DependenciesContract>,
    pub(in crate::contract) source: Option<SourceContract>,
    pub(in crate::contract) analysis: Option<AnalysisContract>,
    #[serde(default)]
    pub(in crate::contract) profiles: BTreeMap<String, ProfileContract>,
    #[serde(default, rename = "layer")]
    pub(in crate::contract) layers: Vec<LayerContract>,
    #[serde(default, rename = "dependency")]
    pub(in crate::contract) dependency_rules: Vec<DependencyRule>,
    #[serde(default, rename = "scope")]
    pub(in crate::contract) scopes: Vec<ScopeContract>,
    #[serde(default, rename = "owner")]
    pub(in crate::contract) owners: Vec<OwnerContract>,
    #[serde(default, rename = "ratchet")]
    pub(in crate::contract) ratchets: Vec<RatchetContract>,
    #[serde(default, rename = "gate")]
    pub(in crate::contract) gates: Vec<GateContract>,
    #[serde(default, rename = "invariant")]
    pub(in crate::contract) invariants: Vec<InvariantContract>,
}
