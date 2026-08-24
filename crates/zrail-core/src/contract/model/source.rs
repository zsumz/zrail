//! Typed Rust source-role and item-macro authority policy.

use serde::{Deserialize, Serialize};

use crate::contract::{CrateRootSource, MacroBindingMode};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
/// Effective source role selected for one exact path.
pub enum FileRole {
    /// Enforce declarative facade shape and facade size budgets.
    Facade,
    /// Enforce implementation size budgets without facade shape restrictions.
    Implementation,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// One reasoned exact override of an inferred Rust source role.
pub struct FileRoleContract {
    /// Exact normalized repository-relative Rust source path.
    pub path: String,
    /// Effective source role selected for the path.
    pub role: FileRole,
    /// Human justification for overriding the inferred source role.
    pub reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// Reviewed authority for item-producing macro invocations.
pub struct ItemMacroContract {
    /// Macro policy name matched at invocation sites.
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional exact repository-relative Rust source path.
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    /// Optional repository-relative patterns scoping name-level authority.
    pub within: Vec<String>,
    #[serde(
        default,
        rename = "resolution",
        alias = "binding",
        skip_serializing_if = "Option::is_none"
    )]
    /// Optional confidence requirement for provenance-aware binding.
    pub binding: Option<MacroBindingMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional external package provenance required for the macro definition.
    pub source: Option<CrateRootSource>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Exact checked-in expansion manifest for generated namespace bindings.
    pub manifest: Option<String>,
    /// Human justification for accepting generated items at this boundary.
    pub reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// Content-bound exact namespace delta for one item-position macro invocation.
pub struct ItemMacroManifest {
    /// Standalone manifest format; currently `1`.
    pub schema: u64,
    /// Macro identity expected at the selected invocation.
    pub macro_name: String,
    /// SHA-256 of the canonical invocation token stream.
    pub invocation_sha256: String,
    /// Complete generated declarations introduced at the invocation scope.
    #[serde(default, rename = "binding")]
    pub bindings: Vec<ItemMacroBinding>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// One exact generated declaration from an item-macro manifest.
pub struct ItemMacroBinding {
    /// Generated Rust identifier.
    pub name: String,
    /// Namespace shape of the generated declaration.
    pub kind: ItemMacroBindingKind,
    /// Whether the declaration is visible outside its lexical module.
    #[serde(default)]
    pub public: bool,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
/// Supported exact generated declaration namespaces.
pub enum ItemMacroBindingKind {
    /// Type namespace declaration without value construction syntax.
    Type,
    /// Tuple/unit constructor present in both type and value usage.
    Constructor,
    /// Value namespace declaration such as a function, const, or static.
    Value,
}
