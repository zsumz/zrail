//! Workspace paths are normalized before exact declared-versus-observed comparison.

mod dependencies;

use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    path::{Component, Path},
};

use toml::Value;
use zrail_core::{glob_matches, normalize_relative, repository_relative};

use super::{model::Package, parse::CargoModelError};

pub(super) use dependencies::resolve_workspace_dependencies;

pub(super) fn workspace_members(value: &Value) -> Result<Vec<String>, CargoModelError> {
    workspace_patterns(value, "members")
}

pub(super) fn workspace_excludes(value: &Value) -> Result<Vec<String>, CargoModelError> {
    workspace_patterns(value, "exclude")
}

fn workspace_patterns(value: &Value, key: &str) -> Result<Vec<String>, CargoModelError> {
    let Some(workspace) = workspace_table(value)? else {
        return Ok(Vec::new());
    };
    let label = format!("workspace.{key}");
    string_array(workspace.get(key), &label)?
        .into_iter()
        .map(|pattern| normalize_workspace_pattern(&pattern, &label))
        .collect()
}

pub(super) fn workspace_package_edition(value: &Value) -> Result<Option<String>, CargoModelError> {
    let Some(workspace) = workspace_table(value)? else {
        return Ok(None);
    };
    let Some(package) = workspace.get("package") else {
        return Ok(None);
    };
    let package = package
        .as_table()
        .ok_or_else(|| CargoModelError("Cargo [workspace.package] must be a table".into()))?;
    package.get("edition").map_or(Ok(None), |edition| {
        let edition = edition
            .as_str()
            .ok_or_else(|| CargoModelError("workspace.package.edition must be a string".into()))?;
        if !matches!(edition, "2015" | "2018" | "2021" | "2024") {
            return Err(CargoModelError(format!(
                "workspace.package.edition {edition:?} is unsupported"
            )));
        }
        Ok(Some(edition.to_owned()))
    })
}

fn workspace_table(
    value: &Value,
) -> Result<Option<&toml::map::Map<String, Value>>, CargoModelError> {
    value.get("workspace").map_or(Ok(None), |workspace| {
        workspace
            .as_table()
            .map(Some)
            .ok_or_else(|| CargoModelError("Cargo [workspace] must be a table".into()))
    })
}

fn normalize_workspace_pattern(pattern: &str, label: &str) -> Result<String, CargoModelError> {
    let normalized = normalize_relative(Path::new(pattern)).map_err(|error| {
        CargoModelError(format!("{label} path {pattern:?} is invalid: {error}"))
    })?;
    if normalized.is_empty() {
        Ok(".".into())
    } else {
        Ok(normalized)
    }
}

fn string_array(value: Option<&Value>, label: &str) -> Result<Vec<String>, CargoModelError> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    let values = value
        .as_array()
        .ok_or_else(|| CargoModelError(format!("{label} must be an array")))?;
    values
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| CargoModelError(format!("{label} must contain strings")))
        })
        .collect()
}

pub(super) fn normalized_directory(
    root: &Path,
    manifest: &Path,
) -> Result<String, CargoModelError> {
    let relative = manifest
        .parent()
        .map(|path| repository_relative(root, path).map_err(CargoModelError))
        .transpose()?
        .unwrap_or_default();
    if relative.is_empty() {
        Ok(".".into())
    } else {
        Ok(relative)
    }
}

pub(super) fn excluded_member(directory: &str, patterns: &[String]) -> bool {
    patterns
        .iter()
        .any(|pattern| glob_matches(pattern, directory))
}

pub(super) fn expand_members(
    patterns: &[String],
    observed: &[String],
    root_package: bool,
) -> Result<Vec<String>, CargoModelError> {
    let mut expanded = BTreeSet::new();
    if root_package {
        expanded.insert(".".into());
    }
    for pattern in patterns {
        let matching = observed
            .iter()
            .filter(|member| glob_matches(pattern, member))
            .cloned()
            .collect::<Vec<_>>();
        if matching.is_empty() {
            return Err(CargoModelError(format!(
                "workspace member pattern {pattern:?} matched no packages"
            )));
        }
        expanded.extend(matching);
    }
    Ok(expanded.into_iter().collect())
}

pub(super) fn expand_implicit_members(
    explicit: Vec<String>,
    packages: &[Package],
    excludes: &[String],
) -> Result<Vec<String>, CargoModelError> {
    let packages = packages
        .iter()
        .map(|package| (package.directory.as_str(), package))
        .collect::<BTreeMap<_, _>>();
    let mut members = explicit.into_iter().collect::<BTreeSet<_>>();
    let mut queue = members.iter().cloned().collect::<VecDeque<_>>();
    while let Some(directory) = queue.pop_front() {
        let Some(package) = packages.get(directory.as_str()) else {
            continue;
        };
        for dependency in &package.dependencies {
            let Some(target) = dependency.repository_path().map(str::to_owned) else {
                continue;
            };
            if excluded_member(&target, excludes) {
                continue;
            }
            if !packages.contains_key(target.as_str()) {
                return Err(CargoModelError(format!(
                    "path dependency from {:?} names missing package directory {target:?}",
                    package.name
                )));
            }
            if members.insert(target.clone()) {
                queue.push_back(target);
            }
        }
    }
    Ok(members.into_iter().collect())
}

pub(super) fn resolve_inside(
    base: &str,
    relative: &str,
) -> Result<Option<String>, CargoModelError> {
    if relative.contains('\\') {
        return Err(CargoModelError(format!(
            "Cargo dependency path uses a platform-dependent separator: {relative:?}"
        )));
    }
    let path = Path::new(relative);
    if path.is_absolute() {
        return Ok(None);
    }
    let mut parts = if base == "." {
        Vec::new()
    } else {
        base.split('/').map(str::to_owned).collect()
    };
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::Normal(value) => {
                let value = value.to_str().ok_or_else(|| {
                    CargoModelError(format!("Cargo dependency path is not UTF-8: {relative:?}"))
                })?;
                parts.push(value.to_owned());
            }
            Component::ParentDir if parts.pop().is_some() => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => return Ok(None),
        }
    }
    if parts.is_empty() {
        Ok(Some(".".into()))
    } else {
        Ok(Some(parts.join("/")))
    }
}

pub(super) fn nested_boundary_error(origin: Option<&str>, nested: &str) -> CargoModelError {
    let edge = origin.map_or_else(
        || "workspace member".to_owned(),
        |origin| format!("path dependency from {origin:?}"),
    );
    CargoModelError(format!(
        "{edge} crosses from workspace \".\" into nested workspace {nested:?}; multi-workspace dependency resolution is not yet supported"
    ))
}

#[cfg(test)]
#[path = "workspace_test.rs"]
mod workspace_test;
