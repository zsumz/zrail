//! Strict string fields shared by explicit Cargo target declarations.

use toml::Value;

pub(super) fn target_path(
    table: &toml::map::Map<String, Value>,
    default: &str,
) -> Result<String, String> {
    optional_string(table, "path")?.map_or_else(|| Ok(default.into()), Ok)
}

pub(super) fn required_string(
    table: &toml::map::Map<String, Value>,
    key: &str,
    label: &str,
) -> Result<String, String> {
    optional_string(table, key)?.ok_or_else(|| format!("{label} requires {key}"))
}

pub(super) fn optional_string(
    table: &toml::map::Map<String, Value>,
    key: &str,
) -> Result<Option<String>, String> {
    table.get(key).map_or(Ok(None), |value| {
        value
            .as_str()
            .map(|value| Some(value.to_owned()))
            .ok_or_else(|| format!("Cargo target {key} must be a string"))
    })
}
