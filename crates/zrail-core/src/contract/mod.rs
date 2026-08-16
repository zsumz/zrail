//! Strict architecture-contract loading and validation.

mod discover;
mod evidence;
mod hash;
mod imports;
mod load;
mod merge;
mod model;
mod modes;
mod validate;
mod validate_dependencies;
mod validate_evidence;
mod validate_limits;
mod validate_paths;
mod validate_ratchet;
mod validate_sets;
mod validate_source;

#[cfg(test)]
#[path = "validate_fixture_test.rs"]
mod validate_fixture_test;

pub use evidence::{EvidenceReference, parse_evidence_reference};
pub use imports::contract_imports;
pub use load::{
    ContractBundle, ContractError, ContractSource, MAX_CONTRACT_BYTES, MAX_CONTRACT_FILES,
    MAX_IMPORT_DIRECTIVES, load_contract,
};
pub use model::{
    Budget, Contract, CrateRootContract, CrateRootSource, DependenciesContract, DependencyRule,
    EffectBoundary, FileSizeContract, GateContract, GateKind, GeneratedSourceContract,
    HygieneContract, InvariantContract, InvariantStatus, ItemMacroContract, LayerContract,
    LayerDependencies, MacroExpansionAllow, MacroExpansionContract, OutDirSourceContract,
    OwnerContract, ProfileContract, RatchetContract, RepositoryContract, RustSourceContract,
    ScopeContract, SourceContract, SymbolBoundary,
};
pub use modes::{
    CycleMode, DependencyMode, Effect, ExactMode, ExternalDependencyMode, FacadeMode,
    LintSuppressionMode, MacroExpansionMode, MacroInputMode, ModuleDocsMode, OwnerKind, PolicyMode,
    SymlinkMode, TestMode,
};
