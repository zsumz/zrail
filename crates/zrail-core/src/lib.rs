//! Language-neutral architecture contracts, lock state, diagnostics, and semantic diffs.
#![doc = include_str!("crate.md")]
#![deny(missing_docs)]

mod contract;
mod diagnostic;
mod diff;
mod digest;
mod input;
mod lock;
mod path;
mod report;

pub use contract::{
    Budget, Contract, ContractBundle, ContractError, ContractSource, CrateRootContract,
    CrateRootSource, CycleMode, DependenciesContract, DependencyMode, DependencyRule, Effect,
    EffectBoundary, EvidenceReference, ExactMode, ExternalDependencyMode, FacadeMode,
    FileSizeContract, GateContract, GateKind, GeneratedSourceContract, HygieneContract,
    InvariantContract, InvariantStatus, ItemMacroContract, LayerContract, LayerDependencies,
    LintSuppressionMode, MAX_CONTRACT_BYTES, MAX_CONTRACT_FILES, MAX_IMPORT_DIRECTIVES,
    MacroExpansionAllow, MacroExpansionContract, MacroExpansionMode, MacroInputMode,
    ModuleDocsMode, OutDirSourceContract, OwnerContract, OwnerKind, PolicyMode, ProfileContract,
    RatchetContract, RepositoryContract, RustSourceContract, ScopeContract, SourceContract,
    SymbolBoundary, SymlinkMode, TestMode, contract_imports, load_contract,
    parse_evidence_reference,
};
pub use diagnostic::{AnalysisQuality, Finding, FindingSink, Severity, SourceSpan};
pub use diff::{
    ArchitectureChange, ChangeKind, DiffReport, DiffSummary, compare_architecture,
    compare_architecture_checked,
};
pub use digest::sha256_hex;
pub use input::{
    MAX_DIRECTORY_DEPTH, MAX_INPUT_BYTES, MAX_REPOSITORY_ENTRIES, create_text,
    read_bytes_with_limit, read_text, read_text_with_limit, replace_text,
};
pub use lock::{
    LOCK_SCHEMA, LOCK_SEMANTICS, LockError, LockFile, LockedDependency, LockedDependencyKind,
    LockedDependencyScope, LockedDependencySource, LockedGate, LockedGateInput,
    LockedGeneratedSource, LockedMacroImplementation, LockedPackage, LockedRatchet,
};
pub use path::{glob_matches, normalize_relative, repository_file, repository_relative};
pub use report::{Report, ReportStatus, ReportSummary};
