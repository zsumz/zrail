//! Lightweight contract-import discovery for repository-state adapters.

use serde::Deserialize;

use super::ContractError;

#[derive(Debug, Deserialize)]
struct ImportHeader {
    #[serde(default)]
    imports: Vec<String>,
}

/// Reads only the top-level `imports` array from TOML contract source.
///
/// The returned paths retain source order and are not normalized or resolved.
/// Malformed TOML is returned as a [`ContractError`] whose message names
/// `origin`; unknown fields outside the import header are ignored.
pub fn contract_imports(source: &str, origin: &str) -> Result<Vec<String>, ContractError> {
    toml::from_str::<ImportHeader>(source)
        .map(|header| header.imports)
        .map_err(|error| ContractError::one(format!("parse {origin}: {error}")))
}

#[cfg(test)]
#[path = "imports_test.rs"]
mod imports_test;
