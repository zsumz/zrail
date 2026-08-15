//! Strict readers for supported Cargo dependency fields.

use std::collections::BTreeSet;

use toml::Value;

pub(super) fn features(table: &toml::map::Map<String, Value>) -> Result<Vec<String>, String> {
    let Some(value) = table.get("features") else {
        return Ok(Vec::new());
    };
    let values = value
        .as_array()
        .ok_or_else(|| "features must be an array".to_owned())?;
    let mut features = values
        .iter()
        .map(|value| {
            value
                .as_str()
                .filter(|value| !value.is_empty())
                .map(str::to_owned)
                .ok_or_else(|| "features must contain non-empty strings".to_owned())
        })
        .collect::<Result<Vec<_>, _>>()?;
    let unique = features.iter().collect::<BTreeSet<_>>().len();
    if unique != features.len() {
        return Err("features may not contain duplicates".into());
    }
    features.sort();
    Ok(features)
}

pub(super) fn default_features(
    table: &toml::map::Map<String, Value>,
) -> Result<Option<bool>, String> {
    let hyphenated = optional_bool(table, "default-features")?;
    let underscored = optional_bool(table, "default_features")?;
    if hyphenated.is_some() && underscored.is_some() {
        return Err("default-features and default_features may not both be present".into());
    }
    Ok(hyphenated.or(underscored))
}

pub(super) fn optional_string(
    table: &toml::map::Map<String, Value>,
    key: &str,
) -> Result<Option<String>, String> {
    table.get(key).map_or(Ok(None), |value| {
        value
            .as_str()
            .map(|value| Some(value.to_owned()))
            .ok_or_else(|| format!("{key} must be a string"))
    })
}

pub(super) fn optional_bool(
    table: &toml::map::Map<String, Value>,
    key: &str,
) -> Result<Option<bool>, String> {
    table.get(key).map_or(Ok(None), |value| {
        value
            .as_bool()
            .map(Some)
            .ok_or_else(|| format!("{key} must be a boolean"))
    })
}

pub(super) fn nonempty(value: String, label: &str) -> Result<String, String> {
    (!value.trim().is_empty())
        .then_some(value)
        .ok_or_else(|| format!("{label} may not be empty"))
}

pub(super) fn validate_keys(table: &toml::map::Map<String, Value>) -> Result<(), String> {
    const SUPPORTED: &[&str] = &[
        "branch",
        "default-features",
        "default_features",
        "features",
        "git",
        "optional",
        "package",
        "path",
        "registry",
        "registry-index",
        "rev",
        "tag",
        "version",
        "workspace",
    ];
    for key in table.keys() {
        if !SUPPORTED.contains(&key.as_str()) {
            return Err(format!("unsupported dependency field {key:?}"));
        }
    }
    Ok(())
}
