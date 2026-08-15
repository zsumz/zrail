//! Direct dependency extraction across normal, development, build, and target tables.

use std::collections::{BTreeMap, BTreeSet};

use toml::Value;

use super::model::{Dependency, DependencyKind, DependencyPath};

pub(super) fn workspace_dependencies(value: &Value) -> Result<BTreeMap<String, String>, String> {
    let Some(workspace) = value.get("workspace") else {
        return Ok(BTreeMap::new());
    };
    let workspace = workspace
        .as_table()
        .ok_or_else(|| "Cargo [workspace] must be a table".to_owned())?;
    let Some(dependencies) = workspace.get("dependencies") else {
        return Ok(BTreeMap::new());
    };
    let dependencies = dependencies
        .as_table()
        .ok_or_else(|| "Cargo [workspace.dependencies] must be a table".to_owned())?;
    dependencies
        .iter()
        .map(|(alias, value)| {
            dependency_name(alias, value, None)
                .map(|name| (alias.clone(), name))
                .map_err(|error| format!("workspace dependency {alias:?}: {error}"))
        })
        .collect()
}

pub(super) fn collect_dependencies(
    value: &Value,
    workspace: &BTreeMap<String, String>,
) -> Result<Vec<Dependency>, String> {
    let mut result = BTreeSet::new();
    collect_tables(value, workspace, &mut result)?;
    if let Some(targets) = value.get("target") {
        let targets = targets
            .as_table()
            .ok_or_else(|| "Cargo [target] must be a table".to_owned())?;
        for target in targets.values() {
            if !target.is_table() {
                return Err("Cargo target selector must contain a table".into());
            }
            collect_tables(target, workspace, &mut result)?;
        }
    }
    Ok(result
        .into_iter()
        .map(|(name, kind)| Dependency { name, kind })
        .collect())
}

pub(super) fn workspace_dependency_paths(
    value: &Value,
) -> Result<BTreeMap<String, String>, String> {
    let Some(dependencies) = value
        .get("workspace")
        .and_then(Value::as_table)
        .and_then(|workspace| workspace.get("dependencies"))
        .and_then(Value::as_table)
    else {
        return Ok(BTreeMap::new());
    };
    dependencies
        .iter()
        .filter_map(|(alias, value)| {
            value
                .as_table()
                .and_then(|table| table.get("path"))
                .map(|path| {
                    path.as_str()
                        .map(|path| (alias.clone(), path.to_owned()))
                        .ok_or_else(|| {
                            format!("workspace dependency {alias:?} path must be a string")
                        })
                })
        })
        .collect()
}

pub(super) fn collect_dependency_paths(
    value: &Value,
    workspace: &BTreeMap<String, String>,
) -> Result<Vec<DependencyPath>, String> {
    let mut paths = BTreeSet::new();
    collect_path_tables(value, workspace, &mut paths)?;
    if let Some(targets) = value.get("target") {
        let targets = targets
            .as_table()
            .ok_or_else(|| "Cargo [target] must be a table".to_owned())?;
        for target in targets.values() {
            collect_path_tables(target, workspace, &mut paths)?;
        }
    }
    Ok(paths
        .into_iter()
        .map(|(path, workspace_relative)| DependencyPath {
            path,
            workspace_relative,
        })
        .collect())
}

fn collect_path_tables(
    value: &Value,
    workspace: &BTreeMap<String, String>,
    paths: &mut BTreeSet<(String, bool)>,
) -> Result<(), String> {
    for key in ["dependencies", "dev-dependencies", "build-dependencies"] {
        let Some(table) = value.get(key) else {
            continue;
        };
        let table = table
            .as_table()
            .ok_or_else(|| format!("Cargo [{key}] must be a table"))?;
        for (alias, spec) in table {
            let Some(spec) = spec.as_table() else {
                continue;
            };
            if spec.get("workspace").and_then(Value::as_bool) == Some(true) {
                if let Some(path) = workspace.get(alias) {
                    paths.insert((path.clone(), true));
                }
            } else if let Some(path) = spec.get("path") {
                let path = path
                    .as_str()
                    .ok_or_else(|| format!("dependency {alias:?} path must be a string"))?;
                paths.insert((path.to_owned(), false));
            }
        }
    }
    Ok(())
}

fn collect_tables(
    value: &Value,
    workspace: &BTreeMap<String, String>,
    result: &mut BTreeSet<(String, DependencyKind)>,
) -> Result<(), String> {
    for (key, kind) in [
        ("dependencies", DependencyKind::Normal),
        ("dev-dependencies", DependencyKind::Development),
        ("build-dependencies", DependencyKind::Build),
    ] {
        collect_dependency_table(value, key, kind, workspace, result)?;
    }
    Ok(())
}

fn collect_dependency_table(
    value: &Value,
    key: &str,
    kind: DependencyKind,
    workspace: &BTreeMap<String, String>,
    result: &mut BTreeSet<(String, DependencyKind)>,
) -> Result<(), String> {
    let Some(table) = value.get(key) else {
        return Ok(());
    };
    let table = table
        .as_table()
        .ok_or_else(|| format!("Cargo [{key}] must be a table"))?;
    for (alias, spec) in table {
        let name = dependency_name(alias, spec, Some(workspace))
            .map_err(|error| format!("dependency {alias:?}: {error}"))?;
        result.insert((name, kind));
    }
    Ok(())
}

fn dependency_name(
    alias: &str,
    value: &Value,
    workspace: Option<&BTreeMap<String, String>>,
) -> Result<String, String> {
    if value.is_str() {
        return Ok(alias.to_owned());
    }
    let table = value
        .as_table()
        .ok_or_else(|| "specification must be a version string or table".to_owned())?;
    let package = optional_string(table, "package")?;
    let _path = optional_string(table, "path")?;
    let inherited = optional_bool(table, "workspace")?;
    if inherited == Some(true) {
        if package.is_some() {
            return Err("workspace inheritance may not override package".into());
        }
        return workspace
            .and_then(|values| values.get(alias))
            .cloned()
            .ok_or_else(|| "workspace dependency is not declared at the workspace root".into());
    }
    if inherited == Some(false) {
        return Err("workspace inheritance must be true when present".into());
    }
    Ok(package.unwrap_or_else(|| alias.to_owned()))
}

fn optional_string(
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

fn optional_bool(table: &toml::map::Map<String, Value>, key: &str) -> Result<Option<bool>, String> {
    table.get(key).map_or(Ok(None), |value| {
        value
            .as_bool()
            .map(Some)
            .ok_or_else(|| format!("{key} must be a boolean"))
    })
}

#[cfg(test)]
#[path = "dependencies_test.rs"]
mod dependencies_test;
