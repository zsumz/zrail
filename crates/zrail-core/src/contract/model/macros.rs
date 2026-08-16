//! Explicit authority for macro expansion and opaque invocation input.

use serde::{Deserialize, Serialize};

use crate::contract::{CrateRootSource, MacroExpansionMode, MacroInputMode};

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
    #[serde(default)]
    pub inputs: MacroInputMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub definition: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<CrateRootSource>,
    pub reason: String,
}
