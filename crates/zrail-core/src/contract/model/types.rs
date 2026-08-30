//! Exact Rust type-shape and duplication policy.

use serde::{Deserialize, Serialize};

use crate::contract::PolicyReachability;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
/// Rust traits that duplicate a value without consuming it.
pub enum DuplicationTrait {
    /// The standard `Clone` trait.
    Clone,
    /// The standard `Copy` marker trait.
    Copy,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
/// Independently enforceable duplication prohibition for one exact type.
pub enum TypeProhibition {
    /// Reject `#[derive(Clone)]` on the selected type.
    DeriveClone,
    /// Reject `#[derive(Copy)]` on the selected type.
    DeriveCopy,
    /// Reject a manual `Clone` implementation for the selected type.
    ImplClone,
    /// Reject a manual `Copy` implementation for the selected type.
    ImplCopy,
    /// Require every same-package active-world item expansion to attest no duplication effect.
    OpaqueExpansion,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
/// Semantic role of a governed Rust type.
pub enum RustTypeKind {
    #[default]
    /// An exact type without authority-token shape requirements.
    Type,
    /// A private, leaf-module, exactly shaped authority token.
    AuthorityToken,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
/// Whether the modeled Clone/Copy surfaces for a governed type must remain closed.
pub enum CloneCopyPolicy {
    #[default]
    /// No bundled Clone/Copy guarantee beyond explicit prohibitions.
    Allow,
    /// Reject Clone/Copy derives, impls, and opaque same-package active-world expansions.
    Forbidden,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// Repository-wide written Clone/Copy syntax policy.
pub struct RustDuplicationContract {
    #[serde(default = "production_reachability")]
    /// Source reachability covered by written import and macro-token bans.
    pub reachability: PolicyReachability,
    #[serde(default)]
    /// Explicit Clone/Copy imports or renames rejected in governed source.
    pub deny_imports: Vec<DuplicationTrait>,
    #[serde(default)]
    /// Clone/Copy identifiers rejected inside opaque macro token streams.
    pub deny_macro_tokens: Vec<DuplicationTrait>,
}

impl Default for RustDuplicationContract {
    fn default() -> Self {
        Self {
            reachability: PolicyReachability::Production,
            deny_imports: Vec::new(),
            deny_macro_tokens: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// Exact declaration, shape, and duplication policy for one Rust type.
pub struct RustTypeContract {
    /// Stable policy name used in findings, coverage, and semantic diffs.
    pub name: String,
    #[serde(rename = "match")]
    /// Canonical Rust identity of the governed type.
    pub identity: String,
    /// Exact repository-relative source file declaring the type.
    pub path: String,
    #[serde(default)]
    /// Semantic role that determines required shape guarantees.
    pub kind: RustTypeKind,
    #[serde(default)]
    /// Source reachability covered by this type policy.
    pub reachability: PolicyReachability,
    #[serde(default)]
    /// Independently selected prohibitions; mutually exclusive with the forbidden bundle.
    pub deny: Vec<TypeProhibition>,
    #[serde(default)]
    /// Whether every modeled Clone/Copy surface is forbidden together, with an empty `deny` list.
    pub clone_copy: CloneCopyPolicy,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Exact semantic visibility expected on the type declaration.
    pub visibility: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Whether the declaration module must contain no child modules.
    pub leaf_module: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Exact ordered named-field representation, when shape is governed.
    pub fields: Option<Vec<RustFieldContract>>,
    /// Human explanation of the type boundary.
    pub reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// One exact named field in a governed Rust type representation.
pub struct RustFieldContract {
    /// Exact Rust field name.
    pub name: String,
    #[serde(rename = "type")]
    /// Canonical identity of the field's path type.
    pub type_identity: String,
    /// Exact semantic visibility expected on the field.
    pub visibility: String,
}

const fn production_reachability() -> PolicyReachability {
    PolicyReachability::Production
}
