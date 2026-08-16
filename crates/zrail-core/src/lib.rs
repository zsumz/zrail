//! Language-neutral contracts, diagnostics, locks, and architecture diffs.

pub mod contract;
pub mod diagnostic;
pub mod diff;
mod digest;
pub mod input;
pub mod lock;
pub mod path;
pub mod report;

pub use contract::{
    Budget, Contract, ContractBundle, ContractError, CrateRootContract, CrateRootSource, CycleMode,
    DependenciesContract, DependencyMode, DependencyRule, Effect, EffectBoundary,
    EvidenceReference, ExactMode, ExternalDependencyMode, FacadeMode, FileSizeContract,
    GateContract, GateKind, GeneratedSourceContract, HygieneContract, InvariantContract,
    InvariantStatus, ItemMacroContract, LayerContract, LayerDependencies, LintSuppressionMode,
    MAX_CONTRACT_BYTES, MAX_CONTRACT_FILES, MAX_IMPORT_DIRECTIVES, MacroExpansionAllow,
    MacroExpansionContract, MacroExpansionMode, MacroInputMode, ModuleDocsMode,
    OutDirSourceContract, OwnerContract, OwnerKind, PolicyMode, ProfileContract, RatchetContract,
    RepositoryContract, RustSourceContract, ScopeContract, SourceContract, SymbolBoundary,
    SymlinkMode, TestMode, contract_imports, load_contract, parse_evidence_reference,
};
pub use diagnostic::{AnalysisQuality, Finding, FindingSink, Severity, SourceSpan};
pub use diff::{
    ArchitectureChange, ChangeKind, DiffReport, DiffSummary, compare_architecture,
    compare_architecture_checked,
};
pub use digest::sha256_hex;
pub use lock::{
    LOCK_SCHEMA, LOCK_SEMANTICS, LockFile, LockedDependency, LockedDependencyKind,
    LockedDependencyScope, LockedDependencySource, LockedGate, LockedGateInput,
    LockedGeneratedSource, LockedMacroDefinition, LockedMacroImplementation, LockedPackage,
    LockedRatchet,
};
pub use report::{Report, ReportStatus, ReportSummary};
