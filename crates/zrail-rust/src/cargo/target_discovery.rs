//! Cargo filesystem discovery honors edition defaults and portable target names.

use std::{fs, path::Path};

use toml::Value;

pub(super) fn discover_directory(
    package: &Path,
    relative: &str,
) -> Result<Vec<(String, String)>, String> {
    let directory = package.join(relative);
    if !directory.is_dir() {
        return Ok(Vec::new());
    }
    let mut entries = fs::read_dir(&directory)
        .map_err(|error| {
            format!(
                "read Cargo target directory {}: {error}",
                directory.display()
            )
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("read Cargo target entry: {error}"))?;
    entries.sort_by_key(fs::DirEntry::file_name);
    let mut targets = Vec::new();
    for entry in entries {
        let path = entry.path();
        let file_name = entry
            .file_name()
            .into_string()
            .map_err(|_| format!("Cargo target name in {relative:?} is not valid UTF-8"))?;
        if path.is_file() && path.extension().is_some_and(|extension| extension == "rs") {
            let name = path
                .file_stem()
                .and_then(|value| value.to_str())
                .ok_or_else(|| format!("Cargo target {file_name:?} has no UTF-8 file stem"))?;
            targets.push((name.to_owned(), format!("{relative}/{file_name}")));
        } else if path.is_dir() && path.join("main.rs").is_file() {
            targets.push((file_name.clone(), format!("{relative}/{file_name}/main.rs")));
        }
    }
    Ok(targets)
}

pub(super) fn auto_enabled(
    package: &toml::map::Map<String, Value>,
    key: &str,
    default: bool,
) -> Result<bool, String> {
    package.get(key).map_or(Ok(default), |value| {
        value
            .as_bool()
            .ok_or_else(|| format!("package.{key} must be a boolean"))
    })
}

pub(super) fn auto_discovery_default(
    manifest: &Value,
    package: &toml::map::Map<String, Value>,
    workspace_edition: Option<&str>,
) -> Result<bool, String> {
    let edition = match package.get("edition") {
        None => "2015",
        Some(Value::String(edition)) => edition,
        Some(Value::Table(value))
            if value.get("workspace").and_then(Value::as_bool) == Some(true) =>
        {
            workspace_edition.ok_or_else(|| {
                "package.edition inherits missing workspace.package.edition".to_owned()
            })?
        }
        Some(_) => return Err("package.edition must be a string or workspace inheritance".into()),
    };
    if !matches!(edition, "2015" | "2018" | "2021" | "2024") {
        return Err(format!("package.edition {edition:?} is unsupported"));
    }
    let manual = ["lib", "bin", "example", "test", "bench"]
        .iter()
        .any(|key| manifest.get(key).is_some());
    Ok(edition != "2015" || !manual)
}
