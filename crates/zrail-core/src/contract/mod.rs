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

#[cfg(test)]
#[path = "mirror_schema_test.rs"]
mod mirror_schema_test;

pub use evidence::{EvidenceReference, parse_evidence_reference};
pub use imports::contract_imports;
pub use load::{
    ContractBundle, ContractError, ContractSource, MAX_CONTRACT_BYTES, MAX_CONTRACT_FILES,
    MAX_IMPORT_DIRECTIVES, load_contract, load_contract_with_entry,
};
pub use model::{
    AnalysisContract, AnalysisLimits, Budget, CargoFeaturePackageContract,
    CargoFeatureWorldContract, CloneCopyPolicy, Contract, CrateRootContract, CrateRootSource,
    DependenciesContract, DependencyEdgeKind, DependencyReachability, DependencyRule,
    DuplicationTrait, EffectBoundary, FileRole, FileRoleContract, FileSizeContract, GateContract,
    GateKind, GeneratedSourceContract, HygieneContract, InvariantContract, InvariantStatus,
    ItemMacroBinding, ItemMacroBindingKind, ItemMacroContract, ItemMacroManifest, LayerContract,
    LayerDependencies, MAX_TEST_MIRROR_INPUTS, MacroExpansionAllow, MacroExpansionContract,
    OutDirSourceContract, OwnerContract, ProfileContract, RatchetContract, RepositoryContract,
    RustDuplicationContract, RustFieldContract, RustSourceContract, RustTypeContract, RustTypeKind,
    ScopeContract, SourceContract, SymbolBoundary, SyntaxBoundary, TestExecutionIdentity,
    TestMirrorContract, TypeProhibition,
};
pub use modes::{
    AsyncSyntax, CycleMode, DependencyMode, Effect, ExactMode, ExternalDependencyMode, FacadeMode,
    GlobImportMode, LintSuppressionMode, MacroAsyncSyntax, MacroBindingMode,
    MacroDuplicationEffect, MacroExpansionBindings, MacroExpansionMode, MacroFieldMutation,
    MacroInputMode, MacroSourceOperations, ModuleDocsMode, OwnerKind, PolicyMode,
    PolicyReachability, SymlinkMode, TestMode,
};
