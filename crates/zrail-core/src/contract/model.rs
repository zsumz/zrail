//! Typed public schema for `zrail.toml`.

#[path = "model/dependencies.rs"]
mod dependencies;
#[path = "model/evidence.rs"]
mod evidence;

pub use dependencies::{CrateRootContract, DependenciesContract};
pub use evidence::{GateContract, GateKind, InvariantContract, InvariantStatus};

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::modes::{
    Effect, ExactMode, ExternalDependencyMode, FacadeMode, LintSuppressionMode, MacroExpansionMode,
    ModuleDocsMode, OwnerKind, PolicyMode, SymlinkMode, TestMode,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Contract {
    pub schema: u64,
    pub adapters: Vec<String>,
    pub repository: RepositoryContract,
    pub dependencies: DependenciesContract,
    pub source: SourceContract,
    pub profiles: BTreeMap<String, ProfileContract>,
    pub layers: Vec<LayerContract>,
    pub dependency_rules: Vec<DependencyRule>,
    pub scopes: Vec<ScopeContract>,
    pub owners: Vec<OwnerContract>,
    pub ratchets: Vec<RatchetContract>,
    pub gates: Vec<GateContract>,
    pub invariants: Vec<InvariantContract>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RepositoryContract {
    pub roots: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
    pub workspace_members: ExactMode,
    pub nested_git: PolicyMode,
    pub submodules: PolicyMode,
    pub symlinks: SymlinkMode,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SourceContract {
    pub rust: RustSourceContract,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RustSourceContract {
    pub module_docs: ModuleDocsMode,
    pub facades: FacadeMode,
    #[serde(default)]
    pub entrypoints: FacadeMode,
    pub tests: TestMode,
    #[serde(default)]
    pub generated: Vec<GeneratedSourceContract>,
    #[serde(default)]
    pub out_dir: Vec<OutDirSourceContract>,
    #[serde(default)]
    pub item_macros: Vec<ItemMacroContract>,
    #[serde(default)]
    pub macros: MacroExpansionContract,
    pub hygiene: HygieneContract,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<FileSizeContract>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MacroExpansionContract {
    pub mode: MacroExpansionMode,
    #[serde(default)]
    pub allow: Vec<MacroExpansionAllow>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MacroExpansionAllow {
    pub name: String,
    pub reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OutDirSourceContract {
    pub path: String,
    pub output: String,
    pub source: String,
    pub reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GeneratedSourceContract {
    pub root: String,
    pub manifest: String,
    #[serde(default)]
    pub inputs: Vec<String>,
    pub target: usize,
    pub hard: usize,
    pub reason: String,
    #[serde(default)]
    pub auxiliary: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ItemMacroContract {
    pub path: String,
    pub name: String,
    pub reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HygieneContract {
    #[serde(rename = "unsafe")]
    pub unsafe_code: PolicyMode,
    pub lint_suppressions: LintSuppressionMode,
    #[serde(default)]
    pub deny_methods: Vec<String>,
    #[serde(default)]
    pub deny_macros: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Budget {
    pub target: usize,
    pub hard: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FileSizeContract {
    pub facade: Budget,
    pub implementation: Budget,
    pub test: Budget,
    pub auxiliary: Budget,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileContract {
    pub effects: EffectBoundary,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EffectBoundary {
    #[serde(default)]
    pub deny: Vec<Effect>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LayerContract {
    pub name: String,
    pub packages: Vec<String>,
    #[serde(default)]
    pub may_depend_on: Vec<String>,
    #[serde(default)]
    pub profiles: Vec<String>,
    pub reason: String,
    #[serde(default)]
    pub dependencies: LayerDependencies,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LayerDependencies {
    #[serde(default)]
    pub external: ExternalDependencyMode,
}

impl Default for LayerDependencies {
    fn default() -> Self {
        Self {
            external: ExternalDependencyMode::Locked,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DependencyRule {
    pub name: String,
    pub from: String,
    #[serde(default)]
    pub deny: Vec<String>,
    pub reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ScopeContract {
    pub name: String,
    pub include: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
    pub reason: String,
    #[serde(default)]
    pub symbols: SymbolBoundary,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SymbolBoundary {
    #[serde(default)]
    pub deny: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OwnerContract {
    pub name: String,
    pub kind: OwnerKind,
    #[serde(default)]
    pub within: Vec<String>,
    #[serde(rename = "match")]
    pub selector: String,
    pub allow: Vec<String>,
    pub reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RatchetContract {
    pub rule: String,
    pub target: String,
    pub reason: String,
}
