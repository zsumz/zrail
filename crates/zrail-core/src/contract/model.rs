//! Typed public schema for `zrail.toml`.
mod analysis;
mod dependencies;
mod evidence;
mod feature_worlds;
mod macros;
mod policy;
mod repository;
mod source;
mod types;

use super::modes::{
    FacadeMode, GlobImportMode, LintSuppressionMode, ModuleDocsMode, PolicyMode, TestMode,
};
pub use analysis::{AnalysisContract, AnalysisLimits};
pub use dependencies::{
    CrateRootContract, CrateRootSource, DependenciesContract, DependencyEdgeKind,
    DependencyReachability,
};
pub use evidence::{
    GateContract, GateKind, InvariantContract, InvariantStatus, MAX_TEST_MIRROR_INPUTS,
    TestExecutionIdentity, TestMirrorContract,
};
pub use feature_worlds::{CargoFeaturePackageContract, CargoFeatureWorldContract};
pub use macros::{MacroExpansionAllow, MacroExpansionContract};
pub use policy::{
    DependencyRule, EffectBoundary, LayerContract, LayerDependencies, OwnerContract,
    ProfileContract, RatchetContract, ScopeContract, SymbolBoundary, SyntaxBoundary,
};
pub use repository::RepositoryContract;
use serde::{Deserialize, Serialize};
pub use source::{
    FileRole, FileRoleContract, ItemMacroBinding, ItemMacroBindingKind, ItemMacroContract,
    ItemMacroManifest,
};
use std::collections::BTreeMap;
pub use types::{
    CloneCopyPolicy, DuplicationTrait, RustDuplicationContract, RustFieldContract,
    RustTypeContract, RustTypeKind, TypeProhibition,
};

#[rustfmt::skip]
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[doc = "Fully merged and validated architecture policy loaded from `zrail.toml`."] pub struct Contract {
    #[doc = "Contract-format version; the current loader accepts schemas `1` and `2`."] pub schema: u64,
    #[doc = "Language adapters required to analyze the governed repository."] pub adapters: Vec<String>,
    #[doc = "Repository layout and containment policy."] pub repository: RepositoryContract,
    #[doc = "Package dependency topology policy."] pub dependencies: DependenciesContract,
    #[doc = "Language-specific source policy."] pub source: SourceContract,
    #[doc = "Reviewed overrides for input-derived analysis expansion limits."] pub analysis: AnalysisContract,
    #[doc = "Named effect profiles available to architecture layers."] pub profiles: BTreeMap<String, ProfileContract>,
    #[doc = "Ordered package-layer declarations."] pub layers: Vec<LayerContract>,
    #[doc = "Named cross-package dependency prohibitions."] pub dependency_rules: Vec<DependencyRule>,
    #[doc = "Named source scopes used by symbol restrictions."] pub scopes: Vec<ScopeContract>,
    #[doc = "Named allow-list ownership rules for governed source relationships."] pub owners: Vec<OwnerContract>,
    #[doc = "Tightening-only measured limits."] pub ratchets: Vec<RatchetContract>,
    #[doc = "Executable qualification gates."] pub gates: Vec<GateContract>,
    #[doc = "Documented promises with validated evidence references."] pub invariants: Vec<InvariantContract>,
}

#[rustfmt::skip]
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[doc = "Language-specific source policy."] pub struct SourceContract {
    #[doc = "Rust module structure, hygiene, macro, and size policy."] pub rust: RustSourceContract,
}

#[rustfmt::skip]
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[doc = "Rust source architecture policy."] pub struct RustSourceContract {
    #[doc = "Governs module-level documentation."] pub module_docs: ModuleDocsMode,
    #[doc = "Governs declarations and implementation logic in `lib.rs` and `mod.rs` facades."] pub facades: FacadeMode,
    #[serde(default)]
    #[doc = "Governs declarations and implementation logic in `main.rs` entrypoints."] pub entrypoints: FacadeMode,
    #[doc = "Governs placement of unit tests relative to implementation files."] pub tests: TestMode,
    #[serde(default)] #[doc = "Exact reasoned facade and implementation role overrides."] pub file_roles: Vec<FileRoleContract>,
    #[serde(default)] #[doc = "Compiler-owned source trees governed by provenance manifests and budgets."] pub generated: Vec<GeneratedSourceContract>,
    #[serde(default)]
    #[doc = "Build-script output included into source with an explicit authority chain."] pub out_dir: Vec<OutDirSourceContract>,
    #[serde(default)]
    #[doc = "Reviewed item-producing macro invocations."] pub item_macros: Vec<ItemMacroContract>,
    #[serde(default)]
    #[doc = "Exact production-to-test mirrors backed by versioned execution receipts."] pub test_mirrors: Vec<TestMirrorContract>,
    #[serde(default)]
    #[doc = "Exact workspace-wide Cargo feature compilation worlds."] pub feature_worlds: Vec<CargoFeatureWorldContract>,
    #[serde(default)]
    #[doc = "Procedural-macro expansion and input-inspection policy."] pub macros: MacroExpansionContract,
    #[serde(default)]
    #[doc = "Repository-wide written duplication syntax policy."] pub duplication: RustDuplicationContract,
    #[serde(default)]
    #[doc = "Exact per-type shape and non-duplication policies."] pub types: Vec<RustTypeContract>,
    #[doc = "Unsafe-code, lint-suppression, and denied-operation policy."] pub hygiene: HygieneContract,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[doc = "Optional target and hard line budgets by Rust file role."] pub size: Option<FileSizeContract>,
}

#[rustfmt::skip]
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[doc = "Reviewed `OUT_DIR` source inclusion and its generating source."] pub struct OutDirSourceContract {
    #[doc = "Repository-relative Rust file containing the inclusion site."] pub path: String,
    #[doc = "Build output path relative to `OUT_DIR`."] pub output: String,
    #[doc = "Repository-relative generator source responsible for the output."] pub source: String,
    #[doc = "Human justification for accepting generated build output."] pub reason: String,
}

#[rustfmt::skip]
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[doc = "Compiler-owned source tree with locked generation provenance and size limits."] pub struct GeneratedSourceContract {
    #[doc = "Repository-relative root containing generated sources."] pub root: String,
    #[doc = "Repository-relative manifest whose digest records generated provenance."] pub manifest: String,
    #[serde(default)]
    #[doc = "Repository-relative input patterns consumed by the generator."] pub inputs: Vec<String>,
    #[doc = "Advisory generated-file line target."] pub target: usize,
    #[doc = "Maximum permitted generated-file line count."] pub hard: usize,
    #[doc = "Human justification for treating the tree as compiler-owned."] pub reason: String,
    #[serde(default)]
    #[doc = "Generated files with auxiliary semantics and the auxiliary size budget."] pub auxiliary: Vec<String>,
}

#[rustfmt::skip]
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[doc = "Rust source hygiene and denied-operation policy."] pub struct HygieneContract {
    #[serde(rename = "unsafe")]
    #[doc = "Governs use of Rust `unsafe` blocks and items."] pub unsafe_code: PolicyMode,
    #[doc = "Governs source-level lint suppression attributes."] pub lint_suppressions: LintSuppressionMode,
    #[serde(default)]
    #[doc = "Fully qualified method identities rejected at call sites."] pub deny_methods: Vec<String>,
    #[serde(default)]
    #[doc = "Macro names rejected at invocation sites."] pub deny_macros: Vec<String>,
    #[serde(default)]
    #[doc = "Governs written glob imports independently from glob path resolution."] pub glob_imports: GlobImportMode,
}

#[rustfmt::skip]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[doc = "Advisory and enforced line-count thresholds for one Rust file role."] pub struct Budget {
    #[doc = "Advisory threshold whose excess creates architecture debt."] pub target: usize,
    #[doc = "Enforced maximum whose excess is an architecture violation."] pub hard: usize,
}

#[rustfmt::skip]
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[doc = "Rust source line budgets classified by file role."] pub struct FileSizeContract {
    #[doc = "Budget for declarative `lib.rs`, `mod.rs`, and `main.rs` files."] pub facade: Budget,
    #[doc = "Budget for production implementation files."] pub implementation: Budget,
    #[doc = "Budget for sibling and integration test files."] pub test: Budget,
    #[doc = "Budget for build scripts and other auxiliary Rust files."] pub auxiliary: Budget,
}
