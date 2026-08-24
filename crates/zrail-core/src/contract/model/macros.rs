//! Explicit authority for macro expansion and opaque invocation input.

use serde::{Deserialize, Serialize};

use crate::contract::{
    CrateRootSource, MacroBindingMode, MacroExpansionBindings, MacroExpansionMode, MacroInputMode,
};

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// Procedural-macro policy and its reviewed exceptions.
pub struct MacroExpansionContract {
    /// Governs macro expansions that match no allow-list entry.
    pub mode: MacroExpansionMode,
    #[serde(default)]
    /// Reviewed procedural macros permitted by a restrictive mode.
    pub allow: Vec<MacroExpansionAllow>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// Reviewed authority for one procedural macro and its invocation input.
pub struct MacroExpansionAllow {
    /// Cargo-visible macro name matched at invocation sites.
    pub name: String,
    #[serde(default)]
    /// Whether invocations must expose their token input to analysis.
    pub inputs: MacroInputMode,
    #[serde(default, rename = "resolution", alias = "binding")]
    /// Confidence required to bind invocations to this authority.
    pub binding: MacroBindingMode,
    #[serde(default, rename = "namespace_effect", alias = "bindings")]
    /// Whether exact review proves zero ordinary-namespace delta for this expansion.
    pub bindings: MacroExpansionBindings,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional package-qualified macro definition identity.
    pub definition: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional package provenance required for the definition.
    pub source: Option<CrateRootSource>,
    /// Human justification for granting this macro authority.
    pub reason: String,
}
