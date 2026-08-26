//! Language-neutral architecture contracts, lock state, diagnostics, and semantic diffs.
#![doc = include_str!("crate.md")]
#![deny(missing_docs)]

mod contract;
mod contract_edit;
mod diagnostic;
mod diff;
mod digest;
mod input;
mod lock;
mod migration;
mod path;
mod ratchet;
mod receipt;
mod report;

pub use contract::{
    AnalysisContract, AnalysisLimits, AsyncSyntax, Budget, CargoFeaturePackageContract,
    CargoFeatureWorldContract, Contract, ContractBundle, ContractError, ContractSource,
    CrateRootContract, CrateRootSource, CycleMode, DependenciesContract, DependencyEdgeKind,
    DependencyMode, DependencyReachability, DependencyRule, DuplicationTrait, Effect,
    EffectBoundary, EvidenceReference, ExactMode, ExternalDependencyMode, FacadeMode, FileRole,
    FileRoleContract, FileSizeContract, GateContract, GateKind, GeneratedSourceContract,
    GlobImportMode, HygieneContract, InvariantContract, InvariantStatus, ItemMacroBinding,
    ItemMacroBindingKind, ItemMacroContract, ItemMacroManifest, LayerContract, LayerDependencies,
    LintSuppressionMode, MAX_CONTRACT_BYTES, MAX_CONTRACT_FILES, MAX_IMPORT_DIRECTIVES,
    MAX_TEST_MIRROR_INPUTS, MacroAsyncSyntax, MacroBindingMode, MacroDuplicationEffect,
    MacroExpansionAllow, MacroExpansionBindings, MacroExpansionContract, MacroExpansionMode,
    MacroInputMode, ModuleDocsMode, OutDirSourceContract, OwnerContract, OwnerKind, PolicyMode,
    PolicyReachability, ProfileContract, RatchetContract, RepositoryContract,
    RustDuplicationContract, RustFieldContract, RustSourceContract, RustTypeContract, RustTypeKind,
    ScopeContract, SourceContract, SymbolBoundary, SymlinkMode, SyntaxBoundary,
    TestExecutionIdentity, TestMirrorContract, TestMode, TypeLinearity, TypeProhibition,
    contract_imports, load_contract, load_contract_with_entry, parse_evidence_reference,
};
pub use contract_edit::{ContractEditError, format_contract_source, migrate_contract_source};
pub use diagnostic::{
    AnalysisQuality, DiagnosticLimit, Finding, FindingSink, MAX_REPORT_FINDINGS, Severity,
    SourceSpan,
};
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
    LOCK_SCHEMA, LOCK_SEMANTICS, LockError, LockFile, LockedAnalysis, LockedContractSource,
    LockedDependency, LockedDependencyKind, LockedDependencyScope, LockedDependencySource,
    LockedExecutionReceipt, LockedGate, LockedGateInput, LockedGeneratedSource,
    LockedItemMacroManifest, LockedMacroImplementation, LockedMacroSource, LockedPackage,
    LockedRatchet,
};
pub use migration::{
    LockMigrationClassification, LockMigrationEntry, LockMigrationError, LockMigrationReport,
    LockMigrationSummary, compare_lock_epochs,
};
pub use path::{glob_matches, normalize_relative, repository_file, repository_relative};
pub use ratchet::normalize_ratchet_selector;
pub use receipt::{
    EXECUTION_RECEIPT_SCHEMA, ExecutionReceipt, ExecutionReceiptStatus, ExecutionReceiptTest,
    MAX_EXECUTION_RECEIPT_BYTES, MAX_TEST_MIRROR_INPUT_BYTES, parse_execution_receipt,
    test_mirror_input_sha256, validate_execution_receipt, versioned_producer,
};
pub use report::{Report, ReportAnalysis, ReportGroup, ReportStatus, ReportSummary};

#[cfg(test)]
#[path = "lock_analysis_test.rs"]
mod lock_analysis_test;
