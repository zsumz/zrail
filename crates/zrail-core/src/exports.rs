//! Explicit public API re-exports keep the crate facade bounded and declarative.

pub use crate::contract::{
    AnalysisContract, AnalysisLimits, AsyncSyntax, Budget, CargoFeaturePackageContract,
    CargoFeatureWorldContract, CloneCopyPolicy, Contract, ContractBundle, ContractError,
    ContractSource, CrateRootContract, CrateRootSource, CycleMode, DependenciesContract,
    DependencyEdgeKind, DependencyMode, DependencyReachability, DependencyRule, DuplicationTrait,
    Effect, EffectBoundary, EvidenceReference, ExactMode, ExternalDependencyMode, FacadeMode,
    FileRole, FileRoleContract, FileSizeContract, GateContract, GateKind, GeneratedSourceContract,
    GlobImportMode, HygieneContract, InvariantContract, InvariantStatus, ItemMacroBinding,
    ItemMacroBindingKind, ItemMacroContract, ItemMacroManifest, LayerContract, LayerDependencies,
    LintSuppressionMode, LockPackageAssertion, LockPackageIdentity, LockPackageRule,
    MAX_CONTRACT_BYTES, MAX_CONTRACT_FILES, MAX_IMPORT_DIRECTIVES, MAX_TEST_MIRROR_INPUTS,
    MacroAmbientInputs, MacroAsyncSyntax, MacroBindingMode, MacroDuplicationEffect,
    MacroExpansionAllow, MacroExpansionBindings, MacroExpansionContract, MacroExpansionMode,
    MacroFieldMutation, MacroInputMode, MacroSourceOperations, ModuleDocsMode,
    OutDirSourceContract, OwnerContract, OwnerKind, PolicyMode, PolicyReachability,
    ProfileContract, RatchetContract, RepositoryCaseMode, RepositoryContract,
    RepositoryDocumentAssertion, RepositoryDocumentFormat, RepositoryDocumentPredicate,
    RepositoryDocumentValue, RepositoryEntryMode, RepositoryFilePredicate, RepositoryFileRule,
    RepositoryLiteralMode, RepositoryLiteralPredicate, RepositoryNameBasis, RepositoryNamePart,
    RepositoryTextNormalization, RustDuplicationContract, RustFieldContract,
    RustInventoryAssertion, RustInventoryCount, RustInventoryRule, RustInventorySubject,
    RustInventoryWorld, RustSourceContract, RustTypeContract, RustTypeKind, ScopeContract,
    ScopedBudgetContract, SizeExceptionContract, SizeExceptionMetadata, SizePolicyContract,
    SizeRole, SizeTargetMode, SizeThresholds, SourceContract, SymbolBoundary, SymlinkMode,
    SyntaxBoundary, TestExecutionIdentity, TestMirrorContract, TestMode, TypeProhibition,
    contract_imports, load_contract, load_contract_with_entry, parse_evidence_reference,
};
pub use crate::contract_edit::{
    ContractEditError, format_contract_source, migrate_contract_source,
};
pub use crate::diagnostic::{
    AnalysisQuality, DiagnosticLimit, Finding, FindingSink, MAX_REPORT_FINDINGS, Severity,
    SourceSpan,
};
pub use crate::diff::{
    ArchitectureChange, ChangeKind, DiffReport, DiffSummary, compare_architecture,
    compare_architecture_checked,
};
pub use crate::digest::sha256_hex;
pub use crate::input::{
    MAX_DIRECTORY_DEPTH, MAX_INPUT_BYTES, MAX_REPOSITORY_ENTRIES, create_text,
    read_bytes_with_limit, read_text, read_text_with_limit, replace_text,
};
pub use crate::lock::{
    LOCK_SCHEMA, LOCK_SEMANTICS, LockError, LockFile, LockedAnalysis, LockedContractSource,
    LockedDependency, LockedDependencyKind, LockedDependencyScope, LockedDependencySource,
    LockedExecutionReceipt, LockedGate, LockedGateInput, LockedGeneratedSource,
    LockedItemMacroManifest, LockedMacroImplementation, LockedMacroSource, LockedPackage,
    LockedRatchet,
};
pub use crate::migration::{
    LockMigrationBridgeReport, LockMigrationClassification, LockMigrationEntry, LockMigrationError,
    LockMigrationFileChange, LockMigrationFileState, LockMigrationReport, LockMigrationRevision,
    LockMigrationSummary, compare_lock_epochs, compare_lock_epochs_across_revisions,
};
pub use crate::path::{glob_can_match_descendant, glob_matches};
pub use crate::path::{normalize_relative, repository_file, repository_relative};
pub use crate::ratchet::normalize_ratchet_selector;
pub use crate::receipt::{
    EXECUTION_RECEIPT_SCHEMA, ExecutionReceipt, ExecutionReceiptStatus, ExecutionReceiptTest,
    MAX_EXECUTION_RECEIPT_BYTES, MAX_TEST_MIRROR_INPUT_BYTES, parse_execution_receipt,
    test_mirror_input_sha256, validate_execution_receipt, versioned_producer,
};
pub use crate::report::{Report, ReportAnalysis, ReportGroup, ReportStatus, ReportSummary};
