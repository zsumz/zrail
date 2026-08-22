//! Typed public schema for `zrail.toml`.
#[path = "model/dependencies.rs"]
mod dependencies;
#[path = "model/evidence.rs"]
mod evidence;
#[path = "model/macros.rs"]
mod macros;

use super::modes::{
    Effect, ExactMode, ExternalDependencyMode, FacadeMode, LintSuppressionMode, ModuleDocsMode,
    OwnerKind, PolicyMode, SymlinkMode, TestMode,
};
pub use dependencies::{CrateRootContract, CrateRootSource, DependenciesContract};
pub use evidence::{GateContract, GateKind, InvariantContract, InvariantStatus};
pub use macros::{MacroExpansionAllow, MacroExpansionContract};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[rustfmt::skip]
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[doc = "Fully merged and validated architecture policy loaded from `zrail.toml`."] pub struct Contract {
    #[doc = "Contract-format version; the current loader accepts schema `1`."] pub schema: u64,
    #[doc = "Language adapters required to analyze the governed repository."] pub adapters: Vec<String>,
    #[doc = "Repository layout and containment policy."] pub repository: RepositoryContract,
    #[doc = "Package dependency topology policy."] pub dependencies: DependenciesContract,
    #[doc = "Language-specific source policy."] pub source: SourceContract,
    #[doc = "Named effect profiles available to architecture layers."] pub profiles: BTreeMap<String, ProfileContract>,
    #[doc = "Ordered package-layer declarations."] pub layers: Vec<LayerContract>,
    #[doc = "Named cross-package dependency prohibitions."] pub dependency_rules: Vec<DependencyRule>,
    #[doc = "Named source scopes used by symbol restrictions."] pub scopes: Vec<ScopeContract>,
    #[doc = "Named ownership rules for calls, capabilities, or directories."] pub owners: Vec<OwnerContract>,
    #[doc = "Tightening-only measured limits."] pub ratchets: Vec<RatchetContract>,
    #[doc = "Executable qualification gates."] pub gates: Vec<GateContract>,
    #[doc = "Documented promises with validated evidence references."] pub invariants: Vec<InvariantContract>,
}

#[rustfmt::skip]
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[doc = "Repository roots, exclusions, and containment policy."] pub struct RepositoryContract {
    #[doc = "Repository-relative directories included in architecture analysis."] pub roots: Vec<String>,
    #[serde(default)]
    #[doc = "Repository-relative patterns excluded from analysis."] pub exclude: Vec<String>,
    #[doc = "Required relationship between declared and discovered workspace members."] pub workspace_members: ExactMode,
    #[doc = "Policy for nested Git repositories beneath governed roots."] pub nested_git: PolicyMode,
    #[doc = "Policy for Git submodules beneath governed roots."] pub submodules: PolicyMode,
    #[doc = "Policy for symbolic links beneath governed roots."] pub symlinks: SymlinkMode,
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
    #[doc = "Governs implementation logic in `lib.rs` and `mod.rs` facades."] pub facades: FacadeMode,
    #[serde(default)]
    #[doc = "Governs implementation logic in `main.rs` entrypoints."] pub entrypoints: FacadeMode,
    #[doc = "Governs placement of unit tests relative to implementation files."] pub tests: TestMode,
    #[serde(default)]
    #[doc = "Compiler-owned source trees governed by provenance manifests and budgets."] pub generated: Vec<GeneratedSourceContract>,
    #[serde(default)]
    #[doc = "Build-script output included into source with an explicit authority chain."] pub out_dir: Vec<OutDirSourceContract>,
    #[serde(default)]
    #[doc = "Reviewed item-producing macro invocations."] pub item_macros: Vec<ItemMacroContract>,
    #[serde(default)]
    #[doc = "Procedural-macro expansion and input-inspection policy."] pub macros: MacroExpansionContract,
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
#[doc = "Reviewed item-producing macro invocation in repository source."] pub struct ItemMacroContract {
    #[doc = "Repository-relative Rust source path containing the invocation."] pub path: String,
    #[doc = "Macro name as written at the invocation site."] pub name: String,
    #[doc = "Human justification for accepting generated items at this boundary."] pub reason: String,
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

#[rustfmt::skip]
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[doc = "Named capability profile applied to one or more architecture layers."] pub struct ProfileContract {
    #[serde(default)]
    #[doc = "Source reachability to which this profile applies."] pub reachability: super::PolicyReachability,
    #[doc = "Side effects prohibited for packages using this profile."] pub effects: EffectBoundary,
}

#[rustfmt::skip]
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[doc = "Set of side-effect capabilities prohibited by a profile."] pub struct EffectBoundary {
    #[serde(default)]
    #[doc = "Effects rejected when observed in a package using the profile."] pub deny: Vec<Effect>,
}

#[rustfmt::skip]
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[doc = "Named package layer and its permitted dependency boundaries."] pub struct LayerContract {
    #[doc = "Stable layer name referenced by other layer declarations."] pub name: String,
    #[doc = "Cargo package-name patterns assigned to this layer."] pub packages: Vec<String>,
    #[serde(default)]
    #[doc = "Layer names that packages in this layer may depend on."] pub may_depend_on: Vec<String>,
    #[serde(default)]
    #[doc = "Effect-profile names enforced for packages in this layer."] pub profiles: Vec<String>,
    #[doc = "Human explanation of the layer's architectural role."] pub reason: String,
    #[serde(default)]
    #[doc = "Policy for dependencies crossing this layer's repository boundary."] pub dependencies: LayerDependencies,
}

#[rustfmt::skip]
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[doc = "Dependency-source policy attached to an architecture layer."] pub struct LayerDependencies {
    #[serde(default)]
    #[doc = "Authority required for dependencies outside the workspace."] pub external: ExternalDependencyMode,
}

#[rustfmt::skip]
impl Default for LayerDependencies { fn default() -> Self { Self { external: ExternalDependencyMode::Locked } } }

#[rustfmt::skip]
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[doc = "Named prohibition on selected package dependency edges."] pub struct DependencyRule {
    #[doc = "Stable rule name used in findings and semantic diffs."] pub name: String,
    #[doc = "Package-name pattern selecting dependency origins."] pub from: String,
    #[serde(default)]
    #[doc = "Package-name patterns rejected as dependency destinations."] pub deny: Vec<String>,
    #[doc = "Human explanation of the prohibited dependency direction."] pub reason: String,
}

#[rustfmt::skip]
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[doc = "Named repository source scope with optional denied symbols."] pub struct ScopeContract {
    #[doc = "Stable scope name used in findings and semantic diffs."] pub name: String,
    #[doc = "Repository-relative patterns included in this scope."] pub include: Vec<String>,
    #[serde(default)]
    #[doc = "Repository-relative patterns removed from the included set."] pub exclude: Vec<String>,
    #[doc = "Human explanation of the scope boundary."] pub reason: String,
    #[serde(default)]
    #[doc = "Symbols prohibited within the resulting source set."] pub symbols: SymbolBoundary,
}

#[rustfmt::skip]
#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[doc = "Denied symbol identities applied within a source scope."] pub struct SymbolBoundary {
    #[serde(default)]
    #[doc = "Fully qualified symbols rejected when used in the scope."] pub deny: Vec<String>,
}

#[rustfmt::skip]
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[doc = "Allow-list boundary for a selected call, capability, or directory owner."] pub struct OwnerContract {
    #[doc = "Stable owner-rule name used in findings and semantic diffs."] pub name: String,
    #[doc = "Kind of source relationship selected by this rule."] pub kind: OwnerKind,
    #[serde(default)] #[doc = "Source reachability to which call and capability ownership applies."] pub reachability: super::PolicyReachability,
    #[serde(default)] #[doc = "Repository-relative patterns limiting where the rule is evaluated."] pub within: Vec<String>,
    #[serde(rename = "match")]
    #[doc = "Call, capability, or directory identity governed by this rule."] pub selector: String,
    #[doc = "Package or source identities permitted to own the selected boundary."] pub allow: Vec<String>,
    #[doc = "Human explanation of the ownership boundary."] pub reason: String,
}

#[rustfmt::skip]
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[doc = "Tightening-only measured limit whose accepted value is stored in lock state."] pub struct RatchetContract {
    #[doc = "Measurement rule identifier understood by the active adapter."] pub rule: String,
    #[doc = "Repository-relative or package target measured by the rule."] pub target: String,
    #[doc = "Human explanation of why this metric may only tighten."] pub reason: String,
}
