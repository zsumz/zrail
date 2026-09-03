//! Published manifest identity and library paths must match the selected lock node.

use std::collections::BTreeMap;

use crate::cargo::ResolvedPackageIdentity;

use super::{VerifiedPackage, normalized_relative};

pub(super) fn read(
    identity: &ResolvedPackageIdentity,
    files: BTreeMap<String, String>,
) -> Result<VerifiedPackage, String> {
    let text = files
        .get("Cargo.toml")
        .ok_or_else(|| "crate archive has no Cargo.toml".to_owned())?;
    let value = text
        .parse::<toml::Value>()
        .map_err(|error| format!("crate archive Cargo.toml is invalid: {error}"))?;
    let package = value
        .get("package")
        .and_then(toml::Value::as_table)
        .ok_or_else(|| "crate archive Cargo.toml has no package table".to_owned())?;
    for (field, expected) in [
        ("name", identity.name.as_str()),
        ("version", identity.version.as_str()),
    ] {
        if package.get(field).and_then(toml::Value::as_str) != Some(expected) {
            return Err(format!(
                "crate archive Cargo.toml {field} does not match Cargo.lock"
            ));
        }
    }
    let library = value
        .get("lib")
        .and_then(toml::Value::as_table)
        .and_then(|table| table.get("path"))
        .and_then(toml::Value::as_str)
        .unwrap_or("src/lib.rs");
    let library = normalized_relative(library)?;
    if !files.contains_key(&library) {
        return Err(format!(
            "crate archive library source {library:?} is unavailable"
        ));
    }
    Ok(VerifiedPackage { files, library })
}
