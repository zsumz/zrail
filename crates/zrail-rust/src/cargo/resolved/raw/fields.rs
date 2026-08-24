//! Cargo lock package fields and textual references are validated exactly.

use semver::Version;
use toml::Value;

use crate::cargo::CargoModelError;

use super::RawPackageId;

pub(super) fn package_id(
    table: &toml::map::Map<String, Value>,
) -> Result<RawPackageId, CargoModelError> {
    let name = required_string(table, "name")?;
    let version = required_string(table, "version")?;
    Version::parse(&version).map_err(|error| {
        CargoModelError(format!(
            "Cargo.lock package {name:?} has invalid version: {error}"
        ))
    })?;
    let id = RawPackageId {
        name,
        version,
        source: optional_string(table, "source")?,
    };
    Ok(id)
}

pub(super) fn validate_provenance(
    id: &RawPackageId,
    checksum: Option<&str>,
) -> Result<(), CargoModelError> {
    match id.source.as_deref() {
        Some(source) if source.starts_with("registry+") => {
            if !checksum.is_some_and(valid_checksum) {
                return Err(CargoModelError(format!(
                    "Cargo.lock registry package {} requires a SHA-256 checksum",
                    label(id)
                )));
            }
        }
        Some(source) if source.starts_with("git+") => {
            if checksum.is_some()
                || source
                    .rsplit_once('#')
                    .is_none_or(|(_, revision)| !valid_git_revision(revision))
            {
                return Err(CargoModelError(format!(
                    "Cargo.lock Git package {} requires a precise source revision and no checksum",
                    label(id)
                )));
            }
        }
        Some(source) => {
            return Err(CargoModelError(format!(
                "Cargo.lock package {} has unsupported source {source:?}",
                label(id)
            )));
        }
        None if checksum.is_some() => {
            return Err(CargoModelError(format!(
                "Cargo.lock local package {} may not carry a registry checksum",
                label(id)
            )));
        }
        None => {}
    }
    Ok(())
}

pub(super) fn resolve_reference<'a>(
    reference: &str,
    packages: impl Iterator<Item = &'a RawPackageId>,
) -> Result<RawPackageId, CargoModelError> {
    let (identity, source) = reference
        .strip_suffix(')')
        .and_then(|value| value.rsplit_once(" ("))
        .map_or((reference, None), |(identity, source)| {
            (identity, Some(source))
        });
    let fields = identity.split_whitespace().collect::<Vec<_>>();
    let (name, version) = match fields.as_slice() {
        [name] => (*name, None),
        [name, version] => (*name, Some(*version)),
        _ => {
            return Err(CargoModelError(format!(
                "Cargo.lock dependency reference {reference:?} is malformed"
            )));
        }
    };
    let matches = packages
        .filter(|package| {
            package.name == name
                && version.is_none_or(|value| package.version == value)
                && source.is_none_or(|value| package.source.as_deref() == Some(value))
        })
        .cloned()
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [resolved] => Ok(resolved.clone()),
        [] => Err(CargoModelError(format!(
            "Cargo.lock dependency reference {reference:?} matches no package"
        ))),
        _ => Err(CargoModelError(format!(
            "Cargo.lock dependency reference {reference:?} is ambiguous across {} packages",
            matches.len()
        ))),
    }
}

fn required_string(
    table: &toml::map::Map<String, Value>,
    key: &str,
) -> Result<String, CargoModelError> {
    optional_string(table, key)?
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| CargoModelError(format!("Cargo.lock package requires string {key}")))
}

pub(super) fn optional_string(
    table: &toml::map::Map<String, Value>,
    key: &str,
) -> Result<Option<String>, CargoModelError> {
    table.get(key).map_or(Ok(None), |value| {
        value
            .as_str()
            .map(|value| Some(value.to_owned()))
            .ok_or_else(|| CargoModelError(format!("Cargo.lock package.{key} must be a string")))
    })
}

pub(super) fn string_array(
    value: Option<&Value>,
    field: &str,
) -> Result<Vec<String>, CargoModelError> {
    value.map_or(Ok(Vec::new()), |value| {
        value
            .as_array()
            .ok_or_else(|| CargoModelError(format!("Cargo.lock {field} must be an array")))?
            .iter()
            .map(|item| {
                item.as_str().map(str::to_owned).ok_or_else(|| {
                    CargoModelError(format!("Cargo.lock {field} must contain strings"))
                })
            })
            .collect()
    })
}

fn valid_checksum(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn valid_git_revision(value: &str) -> bool {
    matches!(value.len(), 40 | 64) && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

pub(super) fn label(id: &RawPackageId) -> String {
    format!(
        "{} {} ({})",
        id.name,
        id.version,
        id.source.as_deref().unwrap_or("local")
    )
}
