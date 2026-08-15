//! Lightweight contract-import discovery for repository-state adapters.

use serde::Deserialize;

use super::ContractError;

#[derive(Debug, Deserialize)]
struct ImportHeader {
    #[serde(default)]
    imports: Vec<String>,
}

pub fn contract_imports(source: &str, origin: &str) -> Result<Vec<String>, ContractError> {
    toml::from_str::<ImportHeader>(source)
        .map(|header| header.imports)
        .map_err(|error| ContractError::one(format!("parse {origin}: {error}")))
}

#[cfg(test)]
#[path = "imports_test.rs"]
mod imports_test;
